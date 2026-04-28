use crate::AppState;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

pub async fn process_upscale_job(state: &Arc<AppState>, job: &crate::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = std::time::Instant::now();
    let prompt_settings: crate::prompts::PromptSettings = serde_json::from_value(job.prompt_settings.clone()).unwrap_or_default();

    // 2. Initial Assessment & Pre-processing Pass
    info!("Starting restoration for job {}...", job.id);
    
    // Step 0: Download original
    let raw_bytes = match state.storage.download_object(&job.input_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = state.db.update_job_failed(job.id, &format!("Storage download error: {}", e), start_time.elapsed().as_millis() as i32).await;
            return Err(e);
        }
    };
    
    let (is_low_res, is_grayscale) = {
        use image::GenericImageView;
        let img = image::load_from_memory(&raw_bytes)?;
        let (w, h) = img.dimensions();
        let mp = (w as f32 * h as f32) / 1_000_000.0;
        let gs = crate::processor::is_grayscale(&img);
        info!("Image classification: {}x{} ({:.3} MP), Grayscale: {}", w, h, mp, gs);
        let style = crate::processor::analyze_style(&img, Some(&raw_bytes));
        (mp < 1.0, gs, style)
    };

    let mut current_uri = state.storage.get_signed_url(&job.input_path).await?;
    
    // Step 1: Captioning (via cheap Replicate BLIP)
    let caption = state.replicate.run_blip_caption(&current_uri).await.ok();

    // Step 2: Model Branching
    if prompt_settings.model == "Standard" {
        // --- STANDARD MODE: Dual-pass high-fidelity restoration ---
        // Pass 1: Restore the 'soul' at the model's native sweet spot (1.5K)
        info!("Running Standard Mode Pass 1: Detail restoration (1.5K)...");
        
        let restore_pre_bytes = match crate::processor::scale_to_resolution(&raw_bytes, "1.5K") {
            Ok(b) => b,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Standard restoration scaling error: {}", e), start_time.elapsed().as_millis() as i32).await;
                return Err(e);
            }
        };

        let restore_path = format!("{}/temp/{}_standard_restore.jpg", job.user_id, job.id);
        state.storage.upload_object(&restore_path, restore_pre_bytes, "image/jpeg").await?;
        let restore_uri = state.storage.get_signed_url(&restore_path).await?;

        let restored_uri = match state.replicate.run_p_image_edit(&restore_uri, caption.clone(), &prompt_settings, is_low_res, is_grayscale, false, style).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Standard restoration error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

        // Pass 2: Upscale to final target (2K/4K/6K)
        info!("Running Standard Mode Pass 2: Final {} upscale...", job.quality);
        current_uri = match state.replicate.run_p_image_upscale(&restored_uri, &job.quality, prompt_settings.creativity).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Standard upscale error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };
    } else {
        // --- PREMIUM MODE: Restore (P-Edit) -> Topaz Upscale ---
        info!("Running Premium Mode: Restoration + Topaz pipeline...");
        
        // 1. Restoration Pass (Premium Sweet Spot: 2MP)
        // Topaz works best when the input image is around 2 megapixels.
        let restore_pre_bytes = match crate::processor::scale_to_resolution(&raw_bytes, "2MP") {
            Ok(b) => b,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Premium restoration scaling error: {}", e), start_time.elapsed().as_millis() as i32).await;
                return Err(e);
            }
        };

        let restore_path = format!("{}/temp/{}_premium_restore_input.jpg", job.user_id, job.id);
        state.storage.upload_object(&restore_path, restore_pre_bytes, "image/jpeg").await?;
        let restore_uri = state.storage.get_signed_url(&restore_path).await?;

        let restored_uri = match state.replicate.run_p_image_edit(&restore_uri, caption.clone(), &prompt_settings, is_low_res, is_grayscale, true, style).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Premium restoration error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

        // 2. Final Topaz Upscale
        info!("Running final Topaz upscale...");
        let style = job.style.as_deref().unwrap_or("PHOTOGRAPHY");
        let topaz_mode = prompt_settings.topaz_mode.as_deref().unwrap_or("Standard");
        
        current_uri = match state.replicate.run_topaz(&restored_uri, &job.quality, style, topaz_mode).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Topaz error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };
    }

    let final_url = current_uri;
    let latency_ms = start_time.elapsed().as_millis() as i32;
    let mut usage_json = serde_json::json!({
        "model": prompt_settings.model,
        "refinement": prompt_settings.refinement,
        "creativity": prompt_settings.creativity,
        "quality": job.quality,
    });

    let original_filename = job.prompt_settings.get("original_filename")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
             std::path::Path::new(&job.input_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image")
                .to_string()
        });

    if let Some(fname) = job.prompt_settings.get("original_filename").and_then(|v| v.as_str()) {
        usage_json["original_filename"] = serde_json::json!(fname);
    }

    info!("Pipeline completed for job {}. Updating success.", job.id);
    state.db.update_job_success(job.id, &final_url, &usage_json, latency_ms).await?;

    // 5. Download Output and upload to S3 in background
    spawn_background_upload(state.clone(), job.id, job.user_id, final_url, usage_json, latency_ms, original_filename);

    Ok(())
}

fn spawn_background_upload(
    state: Arc<AppState>,
    job_id: Uuid,
    user_id: Uuid,
    final_url: String,
    usage_json: serde_json::Value,
    latency_ms: i32,
    input_filename: String,
) {
    tokio::spawn(async move {
        info!("Background: Downloading final output for job {}", job_id);
        let client = reqwest::Client::new();
        let bytes = match client.get(&final_url).send().await {
            Ok(r) => match r.bytes().await {
                Ok(b) => b.to_vec(),
                Err(_) => return,
            },
            Err(_) => return,
        };

        let model = usage_json["model"].as_str().unwrap_or("Standard");
        let quality = usage_json["quality"].as_str().unwrap_or("2x");
        let short_id = job_id.to_string().chars().take(8).collect::<String>();
        let clean_name = input_filename.split('.').next().unwrap_or("input")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();

        let processed_filename = format!("{}_{}_{}_{}.png", model, quality, clean_name, short_id);
        let processed_path = format!("{}/processed/{}", user_id, processed_filename);
        let preview_path = format!("{}/processed/{}_thumb.jpg", user_id, short_id);

        info!("Background: Uploading result to S3 for job {}", job_id);
        if state.storage.upload_object(&processed_path, bytes.clone(), "image/png").await.is_err() {
            return;
        }

        let thumb_res = tokio::task::spawn_blocking(move || {
            crate::processor::generate_thumbnail(&bytes)
        }).await.unwrap_or(Err("Panic".into()));

        if let Ok(thumb_data) = thumb_res {
            let _ = state.storage.upload_object(&preview_path, thumb_data, "image/jpeg").await;
        }

        // Update database with permanent S3 path
        let _ = state.db.update_job_success(job_id, &processed_path, &usage_json, latency_ms).await;
        info!("Background: Permanent S3 paths saved for job {}", job_id);
    });
}

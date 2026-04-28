use crate::AppState;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};

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
    
    let (is_low_res, is_grayscale, style, input_mp) = {
        use image::GenericImageView;
        let img = image::load_from_memory(&raw_bytes)?;
        let (w, h) = img.dimensions();
        let mp = (w as f32 * h as f32) / 1_000_000.0;
        let gs = crate::processor::is_grayscale(&img);
        info!("Image classification: {}x{} ({:.3} MP), Grayscale: {}", w, h, mp, gs);
        let style = crate::processor::analyze_style(&img, Some(&raw_bytes));
        (mp < 1.0, gs, style, mp)
    };

    let input_uri = state.storage.get_signed_url(&job.input_path).await?;

    // Step 1: Captioning (Crucial for Golden Prompt)
    info!("Step 1: Generating descriptive caption...");
    let caption = match state.replicate.run_blip_caption(&input_uri).await {
        Ok(cap) => {
            info!("Successfully generated caption: {}", cap);
            Some(cap)
        },
        Err(e) => {
            warn!("Captioning failed (falling back to generic): {}", e);
            None
        }
    };
    
    // Small jitter to stagger bursty Replicate calls
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 2: Model Branching
    let final_url = if prompt_settings.model == "Standard" {
        // Standard Mode: Single-Pass Technical Upscale
        // Strategy: Feed the original image directly if it's under the 2MP memory limit.
        // If larger, downscale to exactly 2MP to stay safe. Never upscale with Lanczos pre-AI.
        let prep_uri = if input_mp <= 2.0 {
            info!("Running Standard Mode: Direct technical upscale (Source: {:.2} MP)", input_mp);
            input_uri.clone()
        } else {
            info!("Running Standard Mode: Downscaling to 2MP for safety (Source: {:.2} MP)", input_mp);
            let prep_bytes = match crate::processor::scale_to_resolution(&raw_bytes, "2MP") {
                Ok(b) => b,
                Err(e) => {
                    let _ = state.db.update_job_failed(job.id, &format!("Scaling error: {}", e), start_time.elapsed().as_millis() as i32).await;
                    let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                    return Err(e);
                }
            };
            match state.storage.upload_temp(prep_bytes, &format!("{}_standard_prep.jpg", job.id)).await {
                Ok(url) => url,
                Err(e) => {
                    let _ = state.db.update_job_failed(job.id, &format!("Storage error: {}", e), start_time.elapsed().as_millis() as i32).await;
                    let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                    return Err(e);
                }
            }
        };

        match state.replicate.run_real_esrgan(&prep_uri, &job.quality).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Standard upscale error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        }
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

        let restore_uri = match state.storage.upload_temp(restore_pre_bytes, &format!("{}_premium_restore.jpg", job.id)).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Storage error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

        let restored_uri = match state.replicate.run_p_image_edit(&restore_uri, caption.clone(), &prompt_settings, is_low_res, is_grayscale, true, style, input_mp).await {
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
        
        match state.replicate.run_topaz(&restored_uri, &job.quality, style, topaz_mode).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Topaz error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        }
    };
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

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
    let input_mp = {
        use image::GenericImageView;
        let img = image::load_from_memory(&raw_bytes)?;
        let (w, h) = img.dimensions();
        let mp = (w as f32 * h as f32) / 1_000_000.0;
        info!("Image size: {}x{} ({:.3} MP)", w, h, mp);
        mp
    };

    let input_uri = state.storage.get_signed_url(&job.input_path).await?;

    // Small jitter to stagger bursty Replicate calls
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 2: Restoration Pass (Optional / Manual)
    let restored_uri = if prompt_settings.restoration_pass {
        info!("Step 2: AI Restoration pass");
        match state.replicate.run_restore_image(&input_uri).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Restoration error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        }
    } else {
        input_uri.clone()
    };

    // Step 3: Topaz Upscale Pipeline (Dynamic Routing)
    // Threshold: 0.6MP. Below this, we use the Dual-Pass strategy (Low Res -> High Fid).
    let final_url = if input_mp < 0.6 {
        // --- DUAL-PASS STRATEGY FOR LOW-RES (< 0.6MP) ---
        info!("Step 3: Running Dual-Pass Topaz for Low-Res image ({:.2} MP)", input_mp);
        
        // Pass 1: Clean up pixelation with Low Resolution V2 (2x)
        let pass1_url = match state.replicate.run_topaz(
            &restored_uri, "2x", "Low Resolution V2", prompt_settings.face_enhancement,
            prompt_settings.noise_reduction, prompt_settings.sharpen, prompt_settings.remove_artifacts
        ).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Topaz Pass 1 error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

        // Pass 2: Final detail with High Fidelity V2 (Remaining factor)
        let remaining_factor = match job.quality.as_str() {
            "4x" | "4K" => "2x", 
            _ => "None",
        };

        if remaining_factor == "None" {
            pass1_url
        } else {
            match state.replicate.run_topaz(
                &pass1_url, remaining_factor, "High Fidelity V2", false,
                prompt_settings.noise_reduction, prompt_settings.sharpen, prompt_settings.remove_artifacts
            ).await {
                Ok(url) => url,
                Err(e) => {
                    let _ = state.db.update_job_failed(job.id, &format!("Topaz Pass 2 error: {}", e), start_time.elapsed().as_millis() as i32).await;
                    let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                    return Err(e);
                }
            }
        }
    } else {
        // --- SINGLE-PASS STRATEGY FOR MID/HIGH-RES (>= 0.6MP) ---
        // Use "Standard V2" for a balanced approach on mid-range images.
        let model = if input_mp < 1.5 { "Standard V2" } else { "High Fidelity V2" };
        info!("Step 3: Running Single-Pass Topaz [{}] for image ({:.2} MP)", model, input_mp);
        
        let factor = match job.quality.as_str() {
            "4x" | "4K" => "4x",
            _ => "2x",
        };
        match state.replicate.run_topaz(
            &restored_uri, factor, model, false,
            prompt_settings.noise_reduction, prompt_settings.sharpen, prompt_settings.remove_artifacts
        ).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Topaz error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        }
    };
    let latency_ms = start_time.elapsed().as_millis() as i32;
    let usage_json = serde_json::json!({
        "quality": job.quality,
        "face_enhancement": prompt_settings.face_enhancement,
        "noise_reduction": prompt_settings.noise_reduction,
        "sharpen": prompt_settings.sharpen,
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

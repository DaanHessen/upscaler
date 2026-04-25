use crate::AppState;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

pub async fn process_upscale_job(state: &Arc<AppState>, job: &crate::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = std::time::Instant::now();
    let prompt_settings: crate::prompts::PromptSettings = serde_json::from_value(job.prompt_settings.clone()).unwrap_or_default();

    // 2. Pre-processing Pass (NAFNet)
    let mut initial_uri = state.storage.get_signed_url(&job.input_path).await?;
    
    let pre_processing = job.prompt_settings.get("pre_processing_actual").and_then(|v| v.as_bool()).unwrap_or(false);
    if pre_processing {
        info!("Running NAFNet pre-processing for job {}", job.id);
        let nafnet_url = match state.replicate.run_nafnet(&initial_uri).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("NAFNet pre-process error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };
        initial_uri = nafnet_url;
    }

    // 3. Topaz Upscale Pass
    info!("Running Topaz Upscale Pass for job {}", job.id);
    
    let megapixels = job.prompt_settings.get("megapixels").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let replicate_scale = match job.quality.as_str() {
        "Auto" => if megapixels < 1.0 { "4x" } else { "2x" },
        "6x" => "6x",
        "4x" => "4x",
        "2x" => "2x",
        _ => "2x",
    };
    
    let style = job.style.as_deref().unwrap_or("PHOTOGRAPHY");
    let topaz_mode = prompt_settings.topaz_mode.as_deref().unwrap_or("Standard");
    let face_enhancement = prompt_settings.face_enhancement;
    
    let mut topaz_url = match state.replicate.run_topaz(&initial_uri, replicate_scale, style, topaz_mode, face_enhancement).await {
        Ok(url) => url,
        Err(e) => {
            let _ = state.db.update_job_failed(job.id, &format!("Topaz error: {}", e), start_time.elapsed().as_millis() as i32).await;
            let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
            return Err(e);
        }
    };
    
    // 4. Optional Post-Upscale Polish Pass (SCUNet)
    let post_polish = job.prompt_settings.get("post_polish_actual").and_then(|v| v.as_bool()).unwrap_or(false);
    if post_polish {
        info!("Running SCUNet post-upscale polish for job {}", job.id);
        topaz_url = match state.replicate.run_scunet(&topaz_url).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("SCUNet polish error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };
    }
    
    let latency_ms = start_time.elapsed().as_millis() as i32;
    let usage_json = serde_json::json!({
        "pre_processing": pre_processing,
        "post_polish": post_polish,
        "topaz_mode": topaz_mode,
        "quality": job.quality,
    });

    // OPTIMIZATION: Update UI immediately with Replicate URL!
    info!("Pipeline completed. Updating DB with Replicate URL for instant UI preview.");
    state.db.update_job_success(job.id, &topaz_url, &usage_json, latency_ms).await?;

    // 5. Download Output and upload to S3 in background
    let state_clone = state.clone();
    let job_id = job.id;
    let user_id = job.user_id;

    tokio::spawn(async move {
        info!("Background: Downloading final output for job {}", job_id);
        let client = reqwest::Client::new();
        let bytes = match client.get(&topaz_url).send().await {
            Ok(r) => match r.bytes().await {
                Ok(b) => b.to_vec(),
                Err(_) => return,
            },
            Err(_) => return,
        };

        let processed_id = Uuid::new_v4();
        let processed_path = format!("{}/processed/{}.png", user_id, processed_id);
        let preview_path = format!("{}/processed/{}_thumb.jpg", user_id, processed_id);

        info!("Background: Uploading result to S3 for job {}", job_id);
        if state_clone.storage.upload_object(&processed_path, bytes.clone(), "image/png").await.is_err() {
            return;
        }

        let thumb_res = tokio::task::spawn_blocking(move || {
            crate::processor::generate_thumbnail(&bytes)
        }).await.unwrap_or(Err("Panic".into()));

        if let Ok(thumb_data) = thumb_res {
            let _ = state_clone.storage.upload_object(&preview_path, thumb_data, "image/jpeg").await;
        }

        // Update database with permanent S3 path
        let _ = state_clone.db.update_job_success(job_id, &processed_path, &usage_json, latency_ms).await;
        info!("Background: Permanent S3 paths saved for job {}", job_id);
    });

    Ok(())
}

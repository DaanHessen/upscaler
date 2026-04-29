use crate::AppState;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

pub async fn process_upscale_job(state: &Arc<AppState>, job: &crate::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = std::time::Instant::now();
    let prompt_settings: crate::prompts::PromptSettings = serde_json::from_value(job.prompt_settings.clone()).unwrap_or_default();

    info!("Starting hybrid pipeline for job {}...", job.id);
    
    // Step 0: Download/Access original
    let raw_bytes = state.storage.download_object(&job.input_path).await?;
    let input_mp = {
        use image::GenericImageView;
        let img = image::load_from_memory(&raw_bytes)?;
        let (w, h) = img.dimensions();
        let mp = (w as f32 * h as f32) / 1_000_000.0;
        info!("Input resolution: {}x{} ({:.3} MP)", w, h, mp);
        mp
    };

    let mut current_uri = state.storage.get_signed_url(&job.input_path).await?;

    // --- STEP 1: PRE-PROCESS PASS (p-image-edit) ---
    if prompt_settings.pre_process_pass {
        // Validation: Only run if resolution is not too small (as per user instruction)
        if input_mp > 0.15 {
             match state.replicate.run_p_image_edit(&current_uri, prompt_settings.creativity).await {
                Ok(url) => current_uri = url,
                Err(e) => {
                    let _ = state.db.update_job_failed(job.id, &format!("Pre-Process error: {}", e), start_time.elapsed().as_millis() as i32).await;
                    let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                    return Err(e);
                }
            }
        } else {
            info!("Skipping Pre-Process pass: input resolution too low ({:.2} MP)", input_mp);
        }
    }

    // --- STEP 2: RESTORATION PASS (restore-image) ---
    if prompt_settings.restoration_pass {
        match state.replicate.run_restore_image(&current_uri).await {
            Ok(url) => current_uri = url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Restoration error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        }
    }

    // --- STEP 3: TOPAZ UPSCALE PIPELINE ---
    // Decision logic for 90% case:
    // < 0.5MP -> Dual Pass (Low Res -> High Fid)
    // > 0.5MP -> Single Pass (Standard V2 or High Fid V2)
    
    let final_url = if input_mp < 0.5 {
        info!("Step 3: Running Dual-Pass Topaz for Low-Res image ({:.2} MP)", input_mp);
        
        let pass1_url = match state.replicate.run_topaz(
            &current_uri, "2x", "Low Resolution V2", prompt_settings.face_enhancement,
            prompt_settings.noise_reduction, prompt_settings.sharpen, prompt_settings.remove_artifacts
        ).await {
            Ok(url) => url,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &format!("Topaz Pass 1 error: {}", e), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

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
        let model = if input_mp < 1.2 { "Standard V2" } else { "High Fidelity V2" };
        info!("Step 3: Running Single-Pass Topaz [{}] for image ({:.2} MP)", model, input_mp);
        
        let factor = match job.quality.as_str() {
            "4x" | "4K" => "4x",
            _ => "2x",
        };
        match state.replicate.run_topaz(
            &current_uri, factor, model, false,
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
        "pre_process": prompt_settings.pre_process_pass,
        "restoration": prompt_settings.restoration_pass,
        "face_enhancement": prompt_settings.face_enhancement,
    });

    let original_filename = prompt_settings.original_filename.clone().unwrap_or_else(|| "image".to_string());

    info!("Pipeline completed for job {}. Updating success.", job.id);
    state.db.update_job_success(job.id, &final_url, &usage_json, latency_ms).await?;

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

        let quality = usage_json["quality"].as_str().unwrap_or("2x");
        let short_id = job_id.to_string().chars().take(8).collect::<String>();
        let clean_name = input_filename.split('.').next().unwrap_or("input")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();

        let processed_filename = format!("Standard_{}_{}_{}.png", quality, clean_name, short_id);
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

        let _ = state.db.update_job_success(job_id, &processed_path, &usage_json, latency_ms).await;
        info!("Background: Permanent S3 paths saved for job {}", job_id);
    });
}

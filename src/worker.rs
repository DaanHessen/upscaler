use crate::AppState;
use crate::models::{Content, GenerateContentRequest, GenerationConfig, ImageConfig, Part, InlineData};
use base64::{engine::general_purpose, Engine as _};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};

pub async fn process_upscale_job(state: &Arc<AppState>, job: &crate::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = std::time::Instant::now();
    let prompt_settings: crate::prompts::PromptSettings = serde_json::from_value(job.prompt_settings.clone()).unwrap_or_default();
    
    // We only download the original image into memory if we need to do the refinement pass, or if debug is on.
    let mut current_image_bytes = Vec::new();

    // 2. Refinement Pass (Gemini)
    if prompt_settings.refinement_pass {
        info!("Downloading original image for Gemini Refinement job {}", job.id);
        current_image_bytes = state.storage.download_object(&job.input_path).await?;

        info!("Running Gemini Refinement Pass for job {}", job.id);
        
        let ratio_name = crate::processor::get_ratio_name(&current_image_bytes)?;
        let base64_data = general_purpose::STANDARD.encode(&current_image_bytes);

        // A strict, conservative prompt to ONLY denoise and remove artifacts
        let system_prompt = "You are an expert image refinement AI. Your sole job is to preprocess the image to reduce artifacts, clean up compression noise, and prepare it for a final upscaling pass. You MUST strictly preserve the exact composition, identity, and important structural features. Do not invent any new subjects, details, or identities. Focus purely on denoising and artifact removal.";
        let user_text = "Carefully remove artifacts and compression noise without changing the core subject, keeping the exact aspect ratio and structure.";

        let request = GenerateContentRequest {
            system_instruction: Some(Content {
                role: "system".to_string(),
                parts: vec![Part { text: Some(system_prompt.to_string()), inline_data: None }],
            }),
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![
                    Part { text: Some(user_text.to_string()), inline_data: None },
                    Part {
                        text: None,
                        inline_data: Some(InlineData { mime_type: "image/jpeg".to_string(), data: base64_data }),
                    },
                ],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["IMAGE".to_string()],
                image_config: Some(ImageConfig {
                    aspect_ratio: ratio_name,
                    image_size: "1K".to_string(), // Use smaller resolution for refinement
                }),
                temperature: Some(0.0), // Always strict for refinement
                thinking_config: None,
                seed: prompt_settings.seed,
            },
        };

        let token_data: String = state.auth.get_token().await?.as_str().to_string();
        
        let mut attempt = 0;
        let gemini_response;
        loop {
            match state.client.generate_image(token_data.as_str(), request.clone()).await {
                Ok(res) => { gemini_response = Ok(res); break; }
                Err(e) => {
                    attempt += 1;
                    if attempt >= 3 { gemini_response = Err(e); break; }
                    warn!("Vertex API error on attempt {}: {}. Retrying in 2 seconds...", attempt, e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }

        let gemini_response = match gemini_response {
            Ok(res) => res,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, &e.to_string(), start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(e);
            }
        };

        let candidate = match gemini_response.candidates.first() {
            Some(c) => c,
            None => {
                let _ = state.db.update_job_failed(job.id, "Gemini returned no candidates", start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err("Gemini returned no candidates".into());
            }
        };

        if candidate.finish_reason == "SAFETY" {
            let _ = state.db.update_job_failed(job.id, "Image rejected by internal safety filters.", start_time.elapsed().as_millis() as i32).await;
            let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
            return Err("Image rejected by internal safety filters.".into());
        }

        let inline_data = match candidate.content.parts.iter().find_map(|p| p.inline_data.as_ref()) {
            Some(d) => d,
            None => {
                let _ = state.db.update_job_failed(job.id, "No image data in Gemini response", start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err("No image data in Gemini response".into());
            }
        };

        let refined_bytes = match general_purpose::STANDARD.decode(&inline_data.data) {
            Ok(b) => b,
            Err(e) => {
                let _ = state.db.update_job_failed(job.id, "Invalid base64 from Gemini", start_time.elapsed().as_millis() as i32).await;
                let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
                return Err(Box::new(e));
            }
        };

        // Safety bypass check (64x64 black image)
        let is_blocked = tokio::task::spawn_blocking({
            let bytes = refined_bytes.clone();
            move || {
                if let Ok(generated_img) = image::load_from_memory(&bytes) {
                    generated_img.width() == 64 && generated_img.height() == 64
                } else { false }
            }
        }).await?;

        if is_blocked {
            let _ = state.db.update_job_failed(job.id, "Image rejected by internal safety filters.", start_time.elapsed().as_millis() as i32).await;
            let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
            return Err("Image rejected by internal safety filters.".into());
        }

        current_image_bytes = refined_bytes;
    }

    if prompt_settings.debug_gemini_only && prompt_settings.refinement_pass {
        info!("DEBUG: Skipping Topaz pass. Returning Gemini output directly for job {}", job.id);
        
        let latency_ms = start_time.elapsed().as_millis() as i32;
        let processed_id = Uuid::new_v4();
        let processed_path = format!("{}/processed/{}.png", job.user_id, processed_id);
        let preview_path = format!("{}/processed/{}_thumb.jpg", job.user_id, processed_id);

        state.storage.upload_object(&processed_path, current_image_bytes.clone(), "image/png").await?;

        let thumb_res = tokio::task::spawn_blocking(move || crate::processor::generate_thumbnail(&current_image_bytes)).await?;
        if let Ok(thumb_data) = thumb_res {
            let _ = state.storage.upload_object(&preview_path, thumb_data, "image/jpeg").await;
        }

        let usage_json = serde_json::json!({"refinement_pass": true, "debug_gemini": true});
        state.db.update_job_success(job.id, &processed_path, &usage_json, latency_ms).await?;
        return Ok(());
    }

    // 3. Topaz Upscale Pass
    info!("Running Topaz Upscale Pass for job {}", job.id);
    let replicate_scale = if job.quality == "4K" { "4x" } else { "2x" };
    let style = job.style.as_deref().unwrap_or("PHOTOGRAPHY");
    
    let input_uri = if prompt_settings.refinement_pass {
        format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(&current_image_bytes))
    } else {
        state.storage.get_signed_url(&job.input_path).await?
    };
    
    let topaz_url = match state.replicate.run_topaz(&input_uri, replicate_scale, style).await {
        Ok(url) => url,
        Err(e) => {
            let _ = state.db.update_job_failed(job.id, &format!("Topaz error: {}", e), start_time.elapsed().as_millis() as i32).await;
            let _ = state.db.refund_credits(job.user_id, job.credits_charged, job.id).await;
            return Err(e);
        }
    };
    
    let latency_ms = start_time.elapsed().as_millis() as i32;
    let usage_json = if prompt_settings.refinement_pass {
        serde_json::json!({"refinement_pass": true})
    } else {
        serde_json::json!({"refinement_pass": false})
    };

    // OPTIMIZATION: Update UI immediately with Replicate URL!
    info!("Topaz completed. Updating DB with Replicate URL for instant UI preview.");
    state.db.update_job_success(job.id, &topaz_url, &usage_json, latency_ms).await?;

    // 4. Download Replicate Output and upload to S3 in background
    let state_clone = state.clone();
    let job_id = job.id;
    let user_id = job.user_id;

    tokio::spawn(async move {
        info!("Background: Downloading Topaz output for job {}", job_id);
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

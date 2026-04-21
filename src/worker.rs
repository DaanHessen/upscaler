use crate::AppState;
use crate::models::{Content, GenerateContentRequest, GenerationConfig, ImageConfig, Part, InlineData};
use crate::processor::{preprocess_image, ResizeMode};
use crate::prompts::build_system_prompt;
use base64::{engine::general_purpose, Engine as _};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};

pub async fn process_upscale_job(state: &Arc<AppState>, job: &crate::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 1. Download original image
    info!("Downloading original image for job {}", job.id);
    let original_data = state.storage.download_object(&job.input_path).await?;

    // 2. Preprocess image
    let processed = tokio::task::spawn_blocking(move || {
        preprocess_image(&original_data, ResizeMode::Pad)
    }).await??;

    let prompt_settings: crate::prompts::PromptSettings = serde_json::from_value(job.prompt_settings.clone()).unwrap_or_default();
    let system_prompt = build_system_prompt(job.style.as_deref().unwrap_or("PHOTOGRAPHY"), &prompt_settings);

    // 3. Get GCP token
    let token_data: String = state.auth.get_token().await?.as_str().to_string();

    // 4. Build and send request to Vertex
    let request = GenerateContentRequest {
        system_instruction: Some(Content {
            role: "system".to_string(),
            parts: vec![Part {
                text: Some(system_prompt.to_string()),
                inline_data: None,
            }],
        }),
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![
                Part {
                    text: Some("Perform super-resolution restore.".to_string()),
                    inline_data: None,
                },
                Part {
                    text: None,
                    inline_data: Some(InlineData {
                        mime_type: "image/jpeg".to_string(),
                        data: processed.base64_data,
                    }),
                },
            ],
        }],
        generation_config: GenerationConfig {
            response_modalities: vec!["IMAGE".to_string()],
            image_config: Some(ImageConfig {
                aspect_ratio: processed.ratio_name,
                image_size: job.quality.clone(),
            }),
            temperature: Some(job.temperature),
            thinking_config: Some(crate::models::ThinkingConfig {
                thinking_level: prompt_settings.thinking_level.clone(),
            }),
        },
    };

    info!("Sending request to Vertex AI for job {} (temp={}, quality={})", job.id, job.temperature, job.quality);
    
    let start_time = std::time::Instant::now();
    let gemini_response = state.client.generate_image(token_data.as_str(), request).await;
    let duration = start_time.elapsed();
    let latency_ms = duration.as_millis() as i32;

    let gemini_response = match gemini_response {
        Ok(res) => res,
        Err(e) => {
            state.db.update_job_failed(job.id, &e.to_string(), latency_ms).await?;
            return Err(e);
        }
    };

    let candidate = gemini_response.candidates.first()
        .ok_or("Gemini returned no candidates")?;

    let inline_data = candidate.content.parts.iter().find_map(|p| p.inline_data.as_ref())
        .ok_or("No image data in Gemini response")?;

    let image_bytes = general_purpose::STANDARD.decode(&inline_data.data)?;

    if candidate.finish_reason == "SAFETY" {
        state.db.update_job_failed(job.id, "Image rejected by internal safety filters.", latency_ms).await?;
        return Err("Image rejected by internal safety filters.".into());
    }

    // Google Vertex AI sometimes returns a 64x64 pure black image bypass instead of explicitly tagging SAFETY
    if let Ok(generated_img) = image::load_from_memory(&image_bytes) {
        if generated_img.width() == 64 && generated_img.height() == 64 {
            state.db.update_job_failed(job.id, "Image rejected by internal safety filters.", latency_ms).await?;
            return Err("Image rejected by internal safety filters.".into());
        }
    }

    // 5. Upload result and generate preview
    let processed_id = Uuid::new_v4();
    let processed_path = format!("{}/processed/{}.png", job.user_id, processed_id);
    let preview_path = format!("{}/processed/{}_thumb.webp", job.user_id, processed_id);

    info!("Uploading result to storage for job {}", job.id);
    state.storage.upload_object(&processed_path, image_bytes.clone(), "image/png").await?;

    // Generate and upload thumbnail for instant history loading
    match crate::processor::generate_thumbnail(&image_bytes) {
        Ok(thumb_data) => {
            info!("Uploading thumbnail to storage for job {}", job.id);
            if let Err(e) = state.storage.upload_object(&preview_path, thumb_data, "image/webp").await {
                warn!("Thumbnail upload failed for job {}: {}", job.id, e);
            }
        },
        Err(e) => warn!("Thumbnail generation failed for job {}: {}", job.id, e),
    }

    // 6. Update database with success
    let usage_json = serde_json::to_value(&gemini_response.usage_metadata).unwrap_or(serde_json::json!({}));
    state.db.update_job_success(job.id, &processed_path, &usage_json, latency_ms).await?;

    Ok(())
}

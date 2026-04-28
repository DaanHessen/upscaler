use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use tracing::info;
use upscaler::{AppState, replicate::ReplicateClient};
use upscaler::prompts::PromptSettings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Required for rustls 0.23+ to select a crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // 1. Setup AppState (bypass DB for stress test if possible, but easier to just init it)
    let config = upscaler::config::Config::load()?;
    let storage = Arc::new(upscaler::storage::StorageService::new().await?);
    let replicate = Arc::new(ReplicateClient::new()?);
    let db = Arc::new(upscaler::db::DbService::new().await?);
    
    // We don't need vertex client or auth for this test
    let state = Arc::new(AppState {
        client: Arc::new(upscaler::client::VertexClient::new(config.project_id.clone(), config.location.clone())),
        replicate: replicate.clone(),
        auth: upscaler::auth::AuthProvider::new().await?,
        storage: storage.clone(),
        db,
        jwks: jsonwebtoken::jwk::JwkSet { keys: vec![] },
        config,
    });

    let input_dir = "stress_test/input";
    let output_dir = "stress_test/output";
    fs::create_dir_all(output_dir).await?;

    let mut entries = fs::read_dir(input_dir).await?;
    let mut count = 0;

    info!("Starting Stress Test Pipeline...");

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() { continue; }
        
        let filename = path.file_name().unwrap().to_str().unwrap();
        info!("Processing: {}", filename);

        let raw_bytes = fs::read(&path).await?;
        
        // Determine is_low_res
        let is_low_res = {
            let img = image::load_from_memory(&raw_bytes)?;
            let (w, h) = img.dimensions();
            let mp = (w as f32 * h as f32) / 1_000_000.0;
            mp < 1.0
        };

        let mode = "Standard";
        let quality = "4x"; // Always 4K
        
        let prompt_settings = PromptSettings {
            model: mode.to_string(),
            creativity: 0.5,
            refinement: true,
            ..Default::default()
        };

        // Upload to storage to get a URI
        let input_path = format!("stress_test/inputs/{}", filename);
        state.storage.upload_object(&input_path, raw_bytes.clone(), "image/jpeg").await?;
        let input_uri = state.storage.get_signed_url(&input_path).await?;

        // Run BLIP captioning
        let caption = state.replicate.run_blip_caption(&input_uri).await.ok();

        // Standard Mode Pipeline: Restore -> P-Upscale
        let res_target = if is_low_res { "1K" } else { "1.5K" };
        let restore_pre_bytes = upscaler::processor::scale_to_resolution(&raw_bytes, res_target)?;
        let restore_path = format!("stress_test/temp/{}_restore.jpg", filename);
        state.storage.upload_object(&restore_path, restore_pre_bytes, "image/jpeg").await?;
        let restore_uri = state.storage.get_signed_url(&restore_path).await?;
        
        let restored_uri = state.replicate.run_p_image_edit(&restore_uri, caption, &prompt_settings, is_low_res, false).await?;
        let final_url = state.replicate.run_p_image_upscale(&restored_uri, quality, 0.5).await?;

        // Download result
        let result_resp = reqwest::get(&final_url).await?;
        let result_bytes = result_resp.bytes().await?;
        let out_filename = format!("{}_{}_{}", mode, quality, filename);
        fs::write(Path::new(output_dir).join(out_filename), result_bytes).await?;
        
        info!("Finished {}: result saved.", filename);
        count += 1;
    }

    info!("Stress test complete. Processed {} images.", count);
    Ok(())
}

use image::GenericImageView;

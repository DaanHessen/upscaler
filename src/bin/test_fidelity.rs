use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use tracing::info;
use upscaler::{AppState, replicate::ReplicateClient};
use upscaler::prompts::PromptSettings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    rustls::crypto::ring::default_provider().install_default().ok();
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = upscaler::config::Config::load()?;
    let storage = Arc::new(upscaler::storage::StorageService::new().await?);
    let replicate = Arc::new(ReplicateClient::new()?);
    let db = Arc::new(upscaler::db::DbService::new().await?);
    
    let state = Arc::new(AppState {
        client: Arc::new(upscaler::client::VertexClient::new(config.project_id.clone(), config.location.clone())),
        replicate: replicate.clone(),
        auth: upscaler::auth::AuthProvider::new().await?,
        storage: storage.clone(),
        db,
        jwks: jsonwebtoken::jwk::JwkSet { keys: vec![] },
        config,
    });

    let filename = "low_1_256x256.jpg";
    let input_path = format!("stress_test/input/{}", filename);
    let raw_bytes = fs::read(&input_path).await?;
    
    let prompt_settings = PromptSettings {
        model: "Standard".to_string(),
        creativity: 0.5,
        refinement: false,
        ..Default::default()
    };

    info!("Running Fidelity Test for {}...", filename);

    // Upload
    let temp_path = format!("stress_test/temp/fidelity_{}", filename);
    state.storage.upload_object(&temp_path, raw_bytes.clone(), "image/jpeg").await?;
    let input_uri = state.storage.get_signed_url(&temp_path).await?;

    let caption = state.replicate.run_blip_caption(&input_uri).await.ok();
    info!("Caption: {:?}", caption);

    let res_target = "1.5K";
    let restore_pre_bytes = upscaler::processor::scale_to_resolution(&raw_bytes, res_target)?;
    let restore_path = format!("stress_test/temp/fidelity_restore_{}", filename);
    state.storage.upload_object(&restore_path, restore_pre_bytes, "image/jpeg").await?;
    let restore_uri = state.storage.get_signed_url(&restore_path).await?;
    
    let restored_uri = state.replicate.run_p_image_edit(&restore_uri, caption, &prompt_settings, true, false, false).await?;
    let final_url = state.replicate.run_p_image_upscale(&restored_uri, "4x", 0.5).await?;

    let resp = reqwest::get(&final_url).await?;
    let bytes = resp.bytes().await?;
    fs::write("stress_test/output_standard/fidelity_low_1.jpg", bytes).await?;
    
    info!("Fidelity test complete. Saved to stress_test/output_standard/fidelity_low_1.jpg");
    Ok(())
}

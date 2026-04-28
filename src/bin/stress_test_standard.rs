use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use tracing::info;
use upscaler::{AppState, replicate::ReplicateClient};
use upscaler::prompts::PromptSettings;
use image::GenericImageView;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Required for rustls 0.23+ to select a crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // 1. Setup AppState
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

    let input_dir = "stress_test/input";
    let output_dir = "stress_test/output_standard";
    fs::create_dir_all(output_dir).await?;

    // Select a subset of images
    let images_to_test = [
        "low_1_256x256.jpg",
        "med_1_1024x1024.jpg",
        "high_1_2048x2048.jpg",
        "ultra_1_3000x2000.jpg",
    ];

    info!("Starting Standard Model Stress Test...");

    for filename in images_to_test {
        let path = Path::new(input_dir).join(filename);
        if !path.exists() {
            info!("Skipping {}, not found.", filename);
            continue;
        }
        
        info!("Processing: {}", filename);
        let raw_bytes = fs::read(&path).await?;
        
        // Analyze image
        let (is_low_res, is_grayscale) = {
            let img = image::load_from_memory(&raw_bytes)?;
            let (w, h) = img.dimensions();
            let mp = (w as f32 * h as f32) / 1_000_000.0;
            let gs = upscaler::processor::is_grayscale(&img);
            info!("  -> {}x{} ({:.3} MP), Grayscale: {}", w, h, mp, gs);
            (mp < 1.0, gs)
        };

        // Test two variants for each image: 
        // 1. Standard (Refinement Off, Creativity 0.5)
        // 2. Refined (Refinement On, Creativity 0.8)
        let variants = [
            ("Standard", false, 0.5),
            ("Refined", true, 0.8),
        ];

        for (variant_name, refinement, creativity) in variants {
            let mode = "Standard";
            let quality = "4x"; 
            
            let prompt_settings = PromptSettings {
                model: mode.to_string(),
                creativity,
                refinement,
                ..Default::default()
            };

            info!("  -> Running variant: {} (Refinement: {}, Creativity: {})", variant_name, refinement, creativity);

            // Upload to storage to get a URI (simulate temporary storage)
            let temp_input_path = format!("stress_test/temp/{}", filename);
            state.storage.upload_object(&temp_input_path, raw_bytes.clone(), "image/jpeg").await?;
            let input_uri = state.storage.get_signed_url(&temp_input_path).await?;

            // Run BLIP captioning
            let caption = state.replicate.run_blip_caption(&input_uri).await.ok();

            // 1. Restoration Pass
            let res_target = "1.5K"; // Using the same target as worker.rs for Standard mode
            let restore_pre_bytes = upscaler::processor::scale_to_resolution(&raw_bytes, res_target)?;
            let restore_path = format!("stress_test/temp/{}_{}_restore.jpg", filename, variant_name);
            state.storage.upload_object(&restore_path, restore_pre_bytes, "image/jpeg").await?;
            let restore_uri = state.storage.get_signed_url(&restore_path).await?;
            
            let restored_uri = state.replicate.run_p_image_edit(&restore_uri, caption, &prompt_settings, is_low_res, is_grayscale, false).await?;
            
            // 2. Upscale Pass
            let final_url = state.replicate.run_p_image_upscale(&restored_uri, quality, creativity).await?;

            // Download result
            let result_resp = reqwest::get(&final_url).await?;
            let result_bytes = result_resp.bytes().await?;
            let out_filename = format!("{}_{}_{}", variant_name, quality, filename);
            fs::write(Path::new(output_dir).join(&out_filename), result_bytes).await?;
            
            info!("    -> Saved: {}", out_filename);
        }
        
        info!("Finished {}: all variants saved.", filename);
    }

    info!("Stress test complete.");
    Ok(())
}

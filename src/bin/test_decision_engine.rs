use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use tracing::info;
use upscaler::{replicate::ReplicateClient};
use image::GenericImageView;

use upscaler::storage::StorageProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Required for rustls 0.23+ to select a crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();
    
    // We want to capture logs manually for the files, but also show them in console
    tracing_subscriber::fmt::init();

    // 1. Setup Services
    let storage = Arc::new(upscaler::storage::StorageService::new().await?);
    let replicate = Arc::new(ReplicateClient::new()?);
    
    let input_dir = "stress_test/input";
    let output_dir = "stress_test/output_decision";
    fs::create_dir_all(output_dir).await?;

    info!("Starting Decision Engine Test...");

    let mut entries = fs::read_dir(input_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|s| s.to_str()) != Some("jpg") {
            continue;
        }

        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        info!("Testing: {}", filename);
        
        let mut test_log = String::new();
        test_log.push_str(&format!("IMAGE: {}\n", filename));
        test_log.push_str("------------------------------------------\n");

        let raw_bytes = fs::read(&path).await?;
        
        // --- 1. LOCAL ANALYSIS ---
        let img = image::load_from_memory(&raw_bytes)?;
        let style_local = upscaler::processor::analyze_style(&img, Some(&raw_bytes));
        
        test_log.push_str(&format!("LOCAL STYLE VERDICT: {:?}\n", style_local));
        test_log.push_str("------------------------------------------\n");

        // --- 2. AI CAPTIONING ---
        let temp_path = format!("test/decision/{}", filename);
        storage.upload_object(&temp_path, raw_bytes.clone(), "image/jpeg").await?;
        let input_uri = storage.get_signed_url(&temp_path).await?;
        
        test_log.push_str("RUNNING BLIP CAPTIONING...\n");
        let caption = match replicate.run_blip_caption(&input_uri).await {
            Ok(c) => {
                test_log.push_str(&format!("AI CAPTION: {}\n", c));
                Some(c)
            },
            Err(e) => {
                test_log.push_str(&format!("AI CAPTION FAILED: {}\n", e));
                None
            }
        };
        test_log.push_str("------------------------------------------\n");

        // --- 3. DECISION ENGINE ---
        // Note: The Decision Engine logic is currently inside ReplicateClient::run_p_image_edit 
        // as a private helper or inline logic. To test it cleanly, I'll use the public methods.
        // I'll call run_p_image_edit but with a "Dry Run" flag if I were to modify the code, 
        // but for now I'll just manually invoke the logic from replicate.rs in this test script.
        
        // Wait, I implemented decide_style_and_category as a private method in ReplicateClient.
        // I'll make it public so I can test it.
        
        let (final_style, category) = replicate.decide_style_and_category(caption.as_deref(), style_local);
        
        test_log.push_str("FINAL DECISION:\n");
        test_log.push_str(&format!("CATEGORY: {}\n", category));
        test_log.push_str(&format!("STYLE: {:?}\n", final_style));
        
        if final_style != style_local {
            test_log.push_str("OVERRIDE: Local classifier corrected by AI context!\n");
        }

        // --- SAVE LOG ---
        let out_path = Path::new(output_dir).join(format!("{}.txt", filename));
        fs::write(out_path, test_log).await?;
        
        info!("  -> Log saved for {}", filename);
    }

    info!("Decision Engine test complete. Results in {}", output_dir);
    Ok(())
}

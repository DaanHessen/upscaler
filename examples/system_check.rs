use upscaler::processor::{init_nsfw, is_nsfw, analyze_style, preprocess_image, ResizeMode};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Initialize Tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("\n--- SYSTEM VERIFICATION CHECK ---\n");

    // 2. Initialize NSFW Model
    println!("Step 1: Initializing NSFW Model...");
    init_nsfw();
    println!("Done.\n");

    // 3. Test NSFW Filtering
    let nsfw_path = "866905-naked-model-on-the-beach-full-frontal-nude-fit_400_400.jpg";
    println!("Step 2: Testing NSFW Filtering on '{}'...", nsfw_path);
    if Path::new(nsfw_path).exists() {
        let data = fs::read(nsfw_path)?;
        let img = image::load_from_memory(&data)?;
        let result = is_nsfw(&img)?;
        println!(">>> Result: NSFW? {} (Expected: true)", result);
        if result {
            println!("[PASS] NSFW filtering is working.");
        } else {
            println!("[FAIL] NSFW filtering failed to detect nudity.");
        }
    } else {
        println!("[ERROR] NSFW test image not found at {}", nsfw_path);
    }
    println!("");

    // 4. Test Style Analysis & Preprocessing on Camel Image
    let camel_path = "Gemini_Generated_Image_dsct5bdsct5bdsct.png";
    println!("Step 3: Testing Style & Preprocessing on '{}'...", camel_path);
    if Path::new(camel_path).exists() {
        let data = fs::read(camel_path)?;
        
        // Test Style Analysis directly
        let img = image::load_from_memory(&data)?;
        let style = analyze_style(&img);
        println!(">>> Result: Style? {:?} (Expected: Photography)", style);
        
        // Test Full Preprocessing
        let processed = preprocess_image(data, ResizeMode::Pad)?;
        println!(">>> Result: Preprocessing success!");
        println!("    - Ratio Name: {}", processed.ratio_name);
        println!("    - Base64 Length: {} characters", processed.base64_data.len());
        println!("    - Detected Style (from preprocess): {:?}", processed.style);
        
        if style == upscaler::processor::ImageStyle::Photography {
            println!("[PASS] Style analysis is working.");
        } else {
            println!("[FAIL] Image was incorrectly classified.");
        }
    } else {
        println!("[ERROR] Camel test image not found at {}", camel_path);
    }

    println!("\n--- VERIFICATION COMPLETE ---");
    Ok(())
}

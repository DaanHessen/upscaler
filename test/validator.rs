use upscaler::processor::{init_nsfw, is_nsfw, analyze_style, ImageStyle};
use std::fs;
use std::path::Path;
use image::DynamicImage;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    println!("\n=== BATCH IMAGE VALIDATION TEST ===\n");

    // Initialize NSFW model
    init_nsfw();

    let test_dir = Path::new("test");
    if !test_dir.exists() || !test_dir.is_dir() {
        return Err("Test directory 'test' not found".into());
    }

    let entries = fs::read_dir(test_dir)?;
    
    let mut results = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if !["jpg", "jpeg", "png", "webp"].contains(&extension.as_str()) {
                continue;
            }

            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            
            // Skip the validator source itself if it's there (though extension check skips it)
            if filename == "validator.rs" {
                continue;
            }

            println!("Processing: {}", filename);

            let data = fs::read(&path)?;
            
            // 1. Load image
            let img = image::load_from_memory(&data)?;
            
            // 2. NSFW Check
            let nsfw_status = is_nsfw(&img)?;
            
            // 3. Style Check
            let style = analyze_style(&img, Some(&data));
            
            results.push((filename.to_string(), nsfw_status, style));
        }
    }

    println!("\n{:<50} | {:<10} | {:<15}", "Filename", "NSFW", "Style");
    println!("{:-<50}-|-{:-<10}-|-{:-<15}", "", "", "");

    let mut nsfw_detected = Vec::new();

    for (filename, nsfw, style) in results {
        let nsfw_str = if nsfw { "YES (!!)" } else { "No" };
        let style_str = match style {
            ImageStyle::Photography => "PHOTOGRAPHY",
            ImageStyle::Illustration => "ILLUSTRATION",
        };
        
        println!("{:<50} | {:<10} | {:<15}", filename, nsfw_str, style_str);
        
        if nsfw {
            nsfw_detected.push(filename);
        }
    }

    println!("\n=== Summary ===");
    println!("NSFW detected in: {:?}", nsfw_detected);
    
    // Check for specific expected NSFW images if they exist
    let expected_nsfw_patterns = ["naked", "nude", "porn", "hentai"];
    let mut missed_nsfw = false;
    
    for entry in fs::read_dir(test_dir)? {
        let path = entry?.path();
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        
        if expected_nsfw_patterns.iter().any(|p| filename.contains(p)) {
            if !nsfw_detected.iter().any(|n| n.to_lowercase() == filename) {
                println!("WARNING: Filename '{}' suggests NSFW but it was NOT flagged!", filename);
                missed_nsfw = true;
            }
        }
    }

    if missed_nsfw {
        println!("\n[FAIL] Some potential NSFW images were missed.");
    } else {
        println!("\n[PASS] All explicitly NSFW-named images were flagged.");
    }

    Ok(())
}

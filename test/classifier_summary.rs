use upscaler::processor::{init_nsfw, is_nsfw, analyze_style, ImageStyle};
use std::fs;
use std::path::Path;
use tracing_subscriber::EnvFilter;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging (muted to focus on our summary)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::WARN.into()))
        .init();

    println!("\n🚀 Starting Image Classification Batch Test...");

    // Initialize NSFW model (if needed, though we primarily check style)
    init_nsfw();

    let test_dir = Path::new("test/images");
    if !test_dir.exists() || !test_dir.is_dir() {
        return Err(format!("Test directory '{:?}' not found", test_dir).into());
    }

    let entries = fs::read_dir(test_dir)?;
    
    let mut results = Vec::new();
    let mut style_counts = HashMap::new();
    let mut nsfw_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if !["jpg", "jpeg", "png", "webp"].contains(&extension.as_str()) {
                continue;
            }

            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            println!("   - Processing {}...", filename);
            
            let data = fs::read(&path)?;
            let img = image::load_from_memory(&data)?;
            
            // NSFW Check
            let is_flagged = is_nsfw(&img)?;
            if is_flagged {
                nsfw_count += 1;
            }
            
            // Style Check
            let style = analyze_style(&img, Some(&data));
            let style_name = match style {
                ImageStyle::Photography => "PHOTOGRAPHY",
                ImageStyle::Illustration => "ILLUSTRATION",
            };
            
            *style_counts.entry(style_name).or_insert(0) += 1;
            
            results.push((filename.to_string(), is_flagged, style_name));
        }
    }

    println!("\n{:<40} | {:<10} | {:<15}", "Filename", "NSFW", "Style");
    println!("{:-<40}-|-{:-<10}-|-{:-<15}", "", "", "");

    for (filename, nsfw, style) in &results {
        let nsfw_str = if *nsfw { "🚩 YES" } else { "✅ No" };
        println!("{:<40} | {:<10} | {:<15}", filename, nsfw_str, style);
    }

    println!("\n📊 === FINAL SUMMARY ===");
    println!("Total Images Processed: {}", results.len());
    println!("NSFW Flagged:          {}", nsfw_count);
    for (style, count) in style_counts {
        println!("{:<22} {}", format!("{}:", style), count);
    }
    println!("========================\n");

    Ok(())
}

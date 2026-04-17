use std::fs;
use std::env;
use upscaler::processor::{init_nsfw, is_nsfw};
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: cargo run --bin single-nsfw-test <image_path>".into());
    }

    let img_path = &args[1];
    println!("Testing NSFW score for: {}", img_path);
    init_nsfw();
    
    let data = fs::read(img_path)?;
    let img = image::load_from_memory(&data)?;
    
    match is_nsfw(&img) {
        Ok(flagged) => println!("Is NSFW? {}", flagged),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

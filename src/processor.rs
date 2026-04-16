use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;
use std::sync::OnceLock;
use nsfw::{create_model, examine, Model};
use tracing::info;

static NSFW_MODEL: OnceLock<Model> = OnceLock::new();

pub fn init_nsfw() {
    let model_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/model.onnx"));
    let model = create_model(&model_bytes[..]).expect("Failed to initialize NSFW model check");
    NSFW_MODEL.set(model).ok();
    info!("NSFW Moderation Model initialized successfully.");
}

pub fn is_nsfw(img: &DynamicImage) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let model = NSFW_MODEL.get().ok_or("NSFW model not initialized")?;
    
    // Parse image
    let rgba = img.to_rgba8();

    let result = examine(model, &rgba).map_err(|e| e.to_string())?;
    
    let mut porn = 0.0;
    let mut hentai = 0.0;

    for class in &result {
        let name_lower = format!("{:?}", class.metric).to_lowercase();
        match name_lower.as_str() {
            "porn" => porn = class.score,
            "hentai" => hentai = class.score,
            _ => {}
        }
    }
    
    info!("NSFW Check - porn: {:.3}, hentai: {:.3}", porn, hentai);
    
    if porn > 0.6 || hentai > 0.6 {
        return Ok(true);
    }
    
    Ok(false)
}

pub enum ResizeMode {
    Pad,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ImageStyle {
    Illustration,
    Photography,
}

struct GeminiRatio {
    name: &'static str,
    target_width: u32,
    target_height: u32,
    ratio: f32,
}

const SUPPORTED_RATIOS: &[GeminiRatio] = &[
    GeminiRatio { name: "1:1", target_width: 1024, target_height: 1024, ratio: 1.0 },
    GeminiRatio { name: "2:3", target_width: 832, target_height: 1248, ratio: 0.6666667 },
    GeminiRatio { name: "3:2", target_width: 1248, target_height: 832, ratio: 1.5 },
    GeminiRatio { name: "3:4", target_width: 864, target_height: 1152, ratio: 0.75 },
    GeminiRatio { name: "4:3", target_width: 1152, target_height: 864, ratio: 1.3333333 },
    GeminiRatio { name: "4:5", target_width: 896, target_height: 1120, ratio: 0.8 },
    GeminiRatio { name: "5:4", target_width: 1120, target_height: 896, ratio: 1.25 },
    GeminiRatio { name: "9:16", target_width: 768, target_height: 1344, ratio: 0.5625 },
    GeminiRatio { name: "16:9", target_width: 1344, target_height: 768, ratio: 1.7777778 },
    GeminiRatio { name: "21:9", target_width: 1536, target_height: 640, ratio: 2.3333333 },
    GeminiRatio { name: "1:4", target_width: 512, target_height: 2048, ratio: 0.25 },
    GeminiRatio { name: "4:1", target_width: 2048, target_height: 512, ratio: 4.0 },
    GeminiRatio { name: "1:8", target_width: 256, target_height: 2048, ratio: 0.125 },
    GeminiRatio { name: "8:1", target_width: 2048, target_height: 256, ratio: 8.0 },
];

pub struct ProcessedImage {
    pub base64_data: String,
    pub ratio_name: String,
    pub style: ImageStyle,
}

pub fn analyze_style(img: &DynamicImage) -> ImageStyle {
    use imageproc::filter::laplacian_filter;
    
    // 1. Convert to grayscale for analysis
    let gray_img = img.to_luma8();
    
    // 2. Apply Laplacian filter to detect edges
    let laplacian = laplacian_filter(&gray_img);
    
    // 3. Calculate Variance
    let pixels: Vec<f32> = laplacian.pixels().map(|p| p.0[0] as f32).collect();
    let n = pixels.len() as f32;
    if n == 0.0 { return ImageStyle::Photography; }
    
    let mean = pixels.iter().sum::<f32>() / n;
    let variance = pixels.iter()
        .map(|&p| (p - mean).powi(2))
        .sum::<f32>() / n;

    info!("Detected Laplacian Variance: {:.2}", variance);

    if variance < 100.0 {
        info!("Classification: ILLUSTRATION");
        ImageStyle::Illustration
    } else {
        info!("Classification: PHOTOGRAPHY");
        ImageStyle::Photography
    }
}

pub fn preprocess_image(
    data: Vec<u8>,
    mode: ResizeMode,
) -> Result<ProcessedImage, Box<dyn Error + Send + Sync>> {
    // 1. Validate MIME with Magic Bytes
    let info = infer::get(&data).ok_or("Unable to determine file format")?;
    if !info.mime_type().starts_with("image/") {
        return Err(format!("Invalid file type: {}. Only images are allowed.", info.mime_type()).into());
    }

    // 2. Load image from memory
    let img = image::load_from_memory(&data)?;
    let (width, height) = img.dimensions();
    let current_ratio = width as f32 / height as f32;
    
    info!("Input image: {}x{} (ratio={:.3})", width, height, current_ratio);

    // 3. Find nearest ratio
    let nearest = SUPPORTED_RATIOS
        .iter()
        .min_by(|a, b| {
            (a.ratio - current_ratio)
                .abs()
                .partial_cmp(&(b.ratio - current_ratio).abs())
                .unwrap()
        })
        .ok_or("No supported ratios found")?;

    info!("Matched ratio: {} (target: {}x{})", nearest.name, nearest.target_width, nearest.target_height);

    // 4. Resize and Pad
    let processed_img = match mode {
        ResizeMode::Pad => {
            let scaled = img.resize(nearest.target_width, nearest.target_height, image::imageops::FilterType::Lanczos3);
            let mut canvas = DynamicImage::new_rgb8(nearest.target_width, nearest.target_height);
            let (sw, sh) = scaled.dimensions();
            let x = (nearest.target_width - sw) / 2;
            let y = (nearest.target_height - sh) / 2;
            image::imageops::replace(&mut canvas, &scaled, x as i64, y as i64);
            canvas
        }
    };

    // 5. Style Analysis
    let style = analyze_style(&img);

    // 6. Encode to Base64 (for Gemini request)
    let mut buffer = Cursor::new(Vec::new());
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 95);
    processed_img.write_with_encoder(jpeg_encoder)?;
    
    let base64_data = general_purpose::STANDARD.encode(buffer.into_inner());

    Ok(ProcessedImage {
        base64_data,
        ratio_name: nearest.name.to_string(),
        style,
    })
}

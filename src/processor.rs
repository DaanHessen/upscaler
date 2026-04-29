use image::{DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;
use std::sync::OnceLock;
use nsfw::{create_model, examine, Model};
use tracing::info;
use exif::{Reader, Tag};

static NSFW_MODEL: OnceLock<Model> = OnceLock::new();

pub fn init_nsfw() {
    let model_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/model.onnx"));
    let model = create_model(&model_bytes[..]).expect("Failed to initialize NSFW model check");
    NSFW_MODEL.set(model).ok();
    info!("NSFW Moderation Model initialized successfully.");
}

pub fn is_nsfw(img: &DynamicImage) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let model = NSFW_MODEL.get().ok_or("NSFW model not initialized")?;
    
    let rgba = img.to_rgba8();
    let result = examine(model, &rgba).map_err(|e| e.to_string())?;
    
    let mut porn = 0.0;
    let mut hentai = 0.0;
    let mut sexy = 0.0;
    let mut neutral = 0.0;
    let mut drawing = 0.0;

    for class in &result {
        let name_lower = format!("{:?}", class.metric).to_lowercase();
        match name_lower.as_str() {
            "porn" => porn = class.score,
            "hentai" => hentai = class.score,
            "sexy" => sexy = class.score,
            "neutral" => neutral = class.score,
            "drawing" | "drawings" => drawing = class.score,
            _ => {}
        }
    }
    
    info!("NSFW scores — porn: {:.3}, hentai: {:.3}, sexy: {:.3}, neutral: {:.3}, drawing: {:.3}", 
        porn, hentai, sexy, neutral, drawing);

    let safe_signal = neutral + drawing;
    let explicit_signal = porn + hentai;

    if porn > 0.99 || hentai > 0.99 {
        info!("NSFW BLOCKED — single category near-certain (porn={:.3}, hentai={:.3})", porn, hentai);
        return Ok(true);
    }

    if explicit_signal > 0.95 && safe_signal < 0.02 {
        info!("NSFW BLOCKED — combined explicit={:.3}, safe={:.3}", explicit_signal, safe_signal);
        return Ok(true);
    }

    info!("NSFW PASSED — explicit={:.3}, safe={:.3}", explicit_signal, safe_signal);
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
    target_width_1mp: u32,
    target_height_1mp: u32,
    target_width_2mp: u32,
    target_height_2mp: u32,
    ratio: f32,
}

// Support 1MP and 2MP resolutions depending on input image size
const SUPPORTED_RATIOS: &[GeminiRatio] = &[
    GeminiRatio { name: "1:1", target_width_1mp: 1024, target_height_1mp: 1024, target_width_2mp: 1440, target_height_2mp: 1440, ratio: 1.0 },
    GeminiRatio { name: "2:3", target_width_1mp: 832, target_height_1mp: 1248, target_width_2mp: 1184, target_height_2mp: 1760, ratio: 0.6666667 },
    GeminiRatio { name: "3:2", target_width_1mp: 1248, target_height_1mp: 832, target_width_2mp: 1760, target_height_2mp: 1184, ratio: 1.5 },
    GeminiRatio { name: "3:4", target_width_1mp: 864, target_height_1mp: 1152, target_width_2mp: 1216, target_height_2mp: 1632, ratio: 0.75 },
    GeminiRatio { name: "4:3", target_width_1mp: 1152, target_height_1mp: 864, target_width_2mp: 1632, target_height_2mp: 1216, ratio: 1.3333333 },
    GeminiRatio { name: "4:5", target_width_1mp: 896, target_height_1mp: 1120, target_width_2mp: 1280, target_height_2mp: 1600, ratio: 0.8 },
    GeminiRatio { name: "5:4", target_width_1mp: 1120, target_height_1mp: 896, target_width_2mp: 1600, target_height_2mp: 1280, ratio: 1.25 },
    GeminiRatio { name: "9:16", target_width_1mp: 768, target_height_1mp: 1344, target_width_2mp: 1088, target_height_2mp: 1888, ratio: 0.5625 },
    GeminiRatio { name: "16:9", target_width_1mp: 1344, target_height_1mp: 768, target_width_2mp: 1888, target_height_2mp: 1088, ratio: 1.7777778 },
    GeminiRatio { name: "21:9", target_width_1mp: 1536, target_height_1mp: 640, target_width_2mp: 2176, target_height_2mp: 896, ratio: 2.3333333 },
    GeminiRatio { name: "1:4", target_width_1mp: 512, target_height_1mp: 2048, target_width_2mp: 704, target_height_2mp: 2880, ratio: 0.25 },
    GeminiRatio { name: "4:1", target_width_1mp: 2048, target_height_1mp: 512, target_width_2mp: 2880, target_height_2mp: 704, ratio: 4.0 },
    GeminiRatio { name: "1:8", target_width_1mp: 256, target_height_1mp: 2048, target_width_2mp: 384, target_height_2mp: 2880, ratio: 0.125 },
    GeminiRatio { name: "8:1", target_width_1mp: 2048, target_height_1mp: 256, target_width_2mp: 2880, target_height_2mp: 384, ratio: 8.0 },
];

pub struct ProcessedImage {
    pub jpeg_bytes: Vec<u8>,
    pub ratio_name: String,
}

/// Euclidean color distance between two RGB pixels. Returns a value 0..441 (sqrt(255²×3)).
#[inline]
fn color_distance(a: &image::Rgb<u8>, b: &image::Rgb<u8>) -> f32 {
    let dr = a.0[0] as f32 - b.0[0] as f32;
    let dg = a.0[1] as f32 - b.0[1] as f32;
    let db = a.0[2] as f32 - b.0[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

pub fn analyze_style(_img: &DynamicImage, _raw_data: Option<&[u8]>) -> ImageStyle {
    ImageStyle::Photography
}

pub fn get_ratio_name(data: &[u8]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let img = image::load_from_memory(data)?;
    let (width, height) = img.dimensions();
    let current_ratio = width as f32 / height as f32;
    let nearest = SUPPORTED_RATIOS
        .iter()
        .min_by(|a, b| {
            (a.ratio - current_ratio)
                .abs()
                .partial_cmp(&(b.ratio - current_ratio).abs())
                .unwrap()
        })
        .ok_or("No supported ratios found")?;
    Ok(nearest.name.to_string())
}

pub fn preprocess_image(
    data: &[u8],
    mode: ResizeMode,
    target_ratio_name: Option<&str>,
) -> Result<ProcessedImage, Box<dyn Error + Send + Sync>> {
    // 1. Validate MIME
    let _info = infer::get(data).ok_or("Unable to determine file format")?;
    
    // 2. Load
    let img = image::load_from_memory(data)?;
    preprocess_image_internal(img, mode, target_ratio_name)
}

pub fn preprocess_image_internal(
    img: DynamicImage,
    mode: ResizeMode,
    target_ratio_name: Option<&str>,
) -> Result<ProcessedImage, Box<dyn Error + Send + Sync>> {
    let (width, height) = img.dimensions();
    let current_ratio = width as f32 / height as f32;
    
    info!("Input image: {}x{} (ratio={:.3})", width, height, current_ratio);

    // 3. Find nearest or requested ratio
    let nearest = if let Some(tr) = target_ratio_name {
        SUPPORTED_RATIOS.iter().find(|r| r.name == tr).ok_or("Invalid target ratio")?
    } else {
        SUPPORTED_RATIOS
            .iter()
            .min_by(|a, b| {
                (a.ratio - current_ratio)
                    .abs()
                    .partial_cmp(&(b.ratio - current_ratio).abs())
                    .unwrap()
            })
            .ok_or("No supported ratios found")?
    };

    // Determine whether to use 1MP or 2MP target dimensions
    let input_pixels = (width as u64) * (height as u64);
    let pixels_1mp = (nearest.target_width_1mp as u64) * (nearest.target_height_1mp as u64);
    let pixels_2mp = (nearest.target_width_2mp as u64) * (nearest.target_height_2mp as u64);

    let dist_1mp = (input_pixels as i64 - pixels_1mp as i64).abs();
    let dist_2mp = (input_pixels as i64 - pixels_2mp as i64).abs();

    let (target_width, target_height) = if dist_2mp < dist_1mp {
        (nearest.target_width_2mp, nearest.target_height_2mp)
    } else {
        (nearest.target_width_1mp, nearest.target_height_1mp)
    };

    info!("Matched ratio: {} (target: {}x{})", nearest.name, target_width, target_height);

    // 4. Resize and Pad
    let processed_img = match mode {
        ResizeMode::Pad => {
            let scaled = img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
            let mut canvas = DynamicImage::new_rgb8(target_width, target_height);
            let (sw, sh) = scaled.dimensions();
            let x = (target_width - sw) / 2;
            let y = (target_height - sh) / 2;
            image::imageops::replace(&mut canvas, &scaled, x as i64, y as i64);
            canvas
        }
    };

    // 5. Encode to JPEG bytes
    let mut buffer = Cursor::new(Vec::new());
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 95);
    processed_img.write_with_encoder(jpeg_encoder)?;

    Ok(ProcessedImage {
        jpeg_bytes: buffer.into_inner(),
        ratio_name: nearest.name.to_string(),
    })
}

/// Generates a lightweight preview thumbnail (Optimized JPEG, max 360px).
pub fn generate_thumbnail(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let img = image::load_from_memory(data)?;
    let (width, height) = img.dimensions();
    
    // Target max dimensions for previews
    const MAX_DIM: u32 = 360;
    
    let (nw, nh) = if width > height {
        if width > MAX_DIM {
            (MAX_DIM, (height as f32 * (MAX_DIM as f32 / width as f32)) as u32)
        } else {
            (width, height)
        }
    } else {
        if height > MAX_DIM {
            ((width as f32 * (MAX_DIM as f32 / height as f32)) as u32, MAX_DIM)
        } else {
            (width, height)
        }
    };

    let thumb = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    
    let mut buffer = Cursor::new(Vec::new());
    // Using high-quality JPEG for fast encoding and small preview size
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 90);
    thumb.write_with_encoder(encoder)?;
    
    Ok(buffer.into_inner())
}

/// Resizes the image to professional resolution standards.
pub fn scale_to_resolution(data: &[u8], resolution: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let img = image::load_from_memory(data)?;
    let (w, h) = img.dimensions();
    
    let scale = match resolution {
        // --- Longest Side Targets (Standard Mode 4K) ---
        "STD_4K" => {
            let target = 3840.0;
            (target / (w.max(h) as f32)).max(1.0)
        }
        // --- Megapixel Targets (Standard & Premium Safety Caps) ---
        "2MP" => {
            let target_pixels = 2_000_000.0;
            let current_pixels = w as f32 * h as f32;
            (target_pixels / current_pixels).sqrt().min(1.0)
        }
        "1.5MP" => {
            let target_pixels = 1_500_000.0;
            let current_pixels = w as f32 * h as f32;
            (target_pixels / current_pixels).sqrt().min(1.0)
        }
        "1.2MP" => {
            let target_pixels = 1_200_000.0;
            let current_pixels = w as f32 * h as f32;
            (target_pixels / current_pixels).sqrt().min(1.0)
        }
        "1.0MP" | "1MP" => {
            let target_pixels = 1_000_000.0;
            let current_pixels = w as f32 * h as f32;
            (target_pixels / current_pixels).sqrt().min(1.0)
        }
        "768px" => {
            let target = 768.0;
            (target / (w.max(h) as f32)).max(1.0)
        }
        // --- Legacy/Short Side Targets (Restore pass) ---
        "1K" | "1024" => (1024.0 / (w.min(h) as f32)).max(1.0),
        "1.5K" | "1536" => (1536.0 / (w.min(h) as f32)).max(1.0),
        "2x" | "2K" => (1080.0 / (w.min(h) as f32)).max(1.0),
        "4x" | "4K" => (2160.0 / (w.min(h) as f32)).max(1.0),
        "6x" | "6K" => (3240.0 / (w.min(h) as f32)).max(1.0),
        _ => (1080.0 / (w.min(h) as f32)).max(1.0),
    };
    
    let nw = (w as f32 * scale).round() as u32;
    let nh = (h as f32 * scale).round() as u32;
    
    // Ensure multiples of 8 for compatibility with AI models (less distortion than 64)
    let nw = (nw / 8) * 8;
    let nh = (nh / 8) * 8;

    info!("Scaling for pipeline ({}): {}x{} -> {}x{}", resolution, w, h, nw, nh);
    let scaled = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    
    let mut buffer = Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 98);
    scaled.write_with_encoder(encoder)?;
    
    Ok(buffer.into_inner())
}
pub fn is_grayscale(img: &image::DynamicImage) -> bool {
    use image::GenericImageView;
    let (w, h) = img.dimensions();
    
    // Sample a few pixels (e.g., 20x20 grid) to check for color variance
    let sample_step_x = (w / 20).max(1);
    let sample_step_y = (h / 20).max(1);
    
    let mut total_variance = 0.0;
    let mut count = 0;
    
    for y in (0..h).step_by(sample_step_y as usize) {
        for x in (0..w).step_by(sample_step_x as usize) {
            let pixel = img.get_pixel(x, y);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            
            // Average distance from grayscale (R=G=B)
            let avg = (r + g + b) / 3.0;
            let variance = (r - avg).abs() + (g - avg).abs() + (b - avg).abs();
            total_variance += variance;
            count += 1;
        }
    }
    
    if count == 0 { return true; }
    let mean_variance = total_variance / count as f32;
    
    // If mean variance per channel is less than 5.0, it's perceptually grayscale
    mean_variance < 5.0
}

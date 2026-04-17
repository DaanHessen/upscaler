use base64::{engine::general_purpose, Engine as _};
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

pub fn analyze_style(img: &DynamicImage, raw_data: Option<&[u8]>) -> ImageStyle {
    use imageproc::filter::laplacian_filter;
    use std::collections::HashSet;
    
    let mut photo_score = 0.0;
    let mut illustration_score = 0.0;

    // 1. Metadata Check (Definitive Signal)
    if let Some(data) = raw_data {
        let mut cursor = std::io::Cursor::new(data);
        if let Ok(reader) = Reader::new().read_from_container(&mut cursor) {
            let mut metadata_found = false;
            let camera_tags = [Tag::Make, Tag::Model, Tag::Software, Tag::DateTime];
            for f in reader.fields() {
                if camera_tags.contains(&f.tag) {
                    metadata_found = true;
                    break;
                }
            }
            if metadata_found {
                info!("Ensemble: EXIF camera metadata detected (+5.0 Photo)");
                photo_score += 5.0;
            }
        }
    }

    // 2. Alpha Channel Check
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        // Check if many pixels have actual transparency
        let has_transparency = rgba.pixels().step_by(10).take(1000).any(|p| p.0[3] < 255);
        if has_transparency {
            info!("Ensemble: Transparency detected (+3.0 Illustration)");
            illustration_score += 3.0;
        }
    }

    // 3. Shannon Entropy (Randomness/Complexity)
    let gray_img = img.to_luma8();
    let mut counts = [0usize; 256];
    for p in gray_img.pixels().step_by(4) {
        counts[p.0[0] as usize] += 1;
    }
    let total_samples = counts.iter().sum::<usize>() as f32;
    let mut entropy = 0.0;
    if total_samples > 0.0 {
        for &count in counts.iter() {
            if count > 0 {
                let p = count as f32 / total_samples;
                entropy -= p * p.log2();
            }
        }
    }

    // 4. Sharpness (Laplacian Variance)
    let laplacian = laplacian_filter(&gray_img);
    let pixels: Vec<f32> = laplacian.pixels().step_by(2).map(|p| p.0[0] as f32).collect();
    let n = pixels.len() as f32;
    let variance = if n > 0.0 {
        let mean = pixels.iter().sum::<f32>() / n;
        pixels.iter().map(|&p| (p - mean).powi(2)).sum::<f32>() / n
    } else {
        0.0
    };

    // 5. Flatness (Digital Cleanliness)
    let (w, h) = img.dimensions();
    let sample_step = (w / 100).max(1).min(h / 100).max(1);
    let mut flat_count = 0;
    let mut samples = 0;
    let rgb = img.to_rgb8();
    for y in (0..h-1).step_by(sample_step as usize) {
        for x in (0..w-1).step_by(sample_step as usize) {
            if rgb.get_pixel(x, y) == rgb.get_pixel(x + 1, y) {
                flat_count += 1;
            }
            samples += 1;
        }
    }
    let flatness = if samples > 0 { flat_count as f32 / samples as f32 } else { 0.0 };

    // 6. Color Diversity
    let mut colors = HashSet::with_capacity(1000);
    for y in (0..h).step_by((sample_step * 2) as usize) {
        for x in (0..w).step_by((sample_step * 2) as usize) {
            colors.insert(rgb.get_pixel(x, y));
            if colors.len() > 5000 { break; } 
        }
    }
    let color_count = colors.len();

    // 7. Scoring Logic V5
    // Entropy is the strongest photo signal (photos > 7.0, art < 6.0)
    if entropy > 7.2 { photo_score += 2.0; }
    if entropy < 6.5 { illustration_score += 1.5; }
    
    // Flatness (The Neutral Zone)
    if flatness > 0.40 { illustration_score += 3.0; } // Heavy illustration
    else if flatness > 0.25 { illustration_score += 1.0; } // Likely illustration/high compression
    else if flatness < 0.08 { photo_score += 1.5; } // Natural grain

    // Combinations
    if color_count > 4000 && entropy > 7.0 { photo_score += 2.0; }
    if color_count < 1000 && flatness > 0.15 { illustration_score += 2.0; }

    info!("Ensemble V5 - Entropy: {:.2}, Flat: {:.2}, Colors: {}, Var: {:.1}", 
        entropy, flatness, color_count, variance);
    info!("Total Scores - Photo: {:.1}, Illustration: {:.1}", photo_score, illustration_score);

    if photo_score >= illustration_score {
        info!("Final Verdict: PHOTOGRAPHY");
        ImageStyle::Photography
    } else {
        info!("Final Verdict: ILLUSTRATION");
        ImageStyle::Illustration
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
    let style = analyze_style(&img, Some(&data));

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

use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;

pub enum ResizeMode {
    Crop,
    Pad,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageStyle {
    Illustration,
    Photography,
}

pub struct ProcessedImage {
    pub base64_data: String,
    pub ratio_name: String,
    pub style: ImageStyle,
}

fn analyze_style(img: &DynamicImage) -> ImageStyle {
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

    println!("Detected Laplacian Variance: {:.2}", variance);

    // Baseline Threshold: 100.0 (per play-book)
    // < 100.0 = Illustration (flat blocks, low gradient change)
    // >= 100.0 = Photography (textures, high gradient change)
    if variance < 100.0 {
        println!("Classification: ILLUSTRATION");
        ImageStyle::Illustration
    } else {
        println!("Classification: PHOTOGRAPHY");
        ImageStyle::Photography
    }
}

pub fn preprocess_image(
    data: Vec<u8>,
    mode: ResizeMode,
) -> Result<ProcessedImage, Box<dyn Error>> {
    // 1. Validate MIME with Magic Bytes
    let info = infer::get(&data).ok_or("Unable to determine file format")?;
    if !info.mime_type().starts_with("image/") {
        return Err(format!("Invalid file type: {}. Only images are allowed.", info.mime_type()).into());
    }

    // 2. Load image from memory
    let img = image::load_from_memory(&data)?;
    let (width, height) = img.dimensions();
    let current_ratio = width as f32 / height as f32;
    
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

    println!("Matched ratio: {} (Target: {}x{})", nearest.name, nearest.target_width, nearest.target_height);

    // 4. Resize and Crop/Pad
    let processed_img = match mode {
        ResizeMode::Crop => {
            let scale_w = nearest.target_width as f32 / width as f32;
            let scale_h = nearest.target_height as f32 / height as f32;
            let scale = scale_w.max(scale_h);

            let new_w = (width as f32 * scale).round() as u32;
            let new_h = (height as f32 * scale).round() as u32;

            let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
            
            let x = (new_w.saturating_sub(nearest.target_width)) / 2;
            let y = (new_h.saturating_sub(nearest.target_height)) / 2;
            
            resized.crop_imm(x, y, nearest.target_width, nearest.target_height)
        }
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

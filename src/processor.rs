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

    // --- Layered Decision Logic ---
    //
    // The local MobileNet classifier (OpenNSFW2) is highly prone to false positives
    // on innocuous close-up skin images (feet, hands, shoulders, swimwear).
    // Vertex AI has its own robust multi-modal safety system as the real gatekeeper.
    // This local filter exists ONLY to save API costs by catching obviously explicit content.
    //
    // Strategy:
    //   1. If neutral + drawing together account for >15% of the signal, it's extremely
    //      unlikely to be actual porn — the model is just confused by skin tones.
    //   2. Only block if a single category is near-certain (>0.99), OR if the combined
    //      explicit signal (porn + hentai) dominates with >0.95 AND the safe categories
    //      are nearly zero.

    let safe_signal = neutral + drawing;
    let explicit_signal = porn + hentai;

    // Hard block: model is overwhelmingly confident in a single explicit category
    if porn > 0.99 || hentai > 0.99 {
        info!("NSFW BLOCKED — single category near-certain (porn={:.3}, hentai={:.3})", porn, hentai);
        return Ok(true);
    }

    // Combined block: strong explicit signal with almost no safe signal
    if explicit_signal > 0.95 && safe_signal < 0.02 {
        info!("NSFW BLOCKED — combined explicit={:.3}, safe={:.3}", explicit_signal, safe_signal);
        return Ok(true);
    }

    // Everything else passes through — Vertex AI will catch anything we miss
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
}

/// Euclidean color distance between two RGB pixels. Returns a value 0..441 (sqrt(255²×3)).
#[inline]
fn color_distance(a: &image::Rgb<u8>, b: &image::Rgb<u8>) -> f32 {
    let dr = a.0[0] as f32 - b.0[0] as f32;
    let dg = a.0[1] as f32 - b.0[1] as f32;
    let db = a.0[2] as f32 - b.0[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

pub fn analyze_style(img: &DynamicImage, raw_data: Option<&[u8]>) -> ImageStyle {
    use std::collections::HashSet;
    
    let mut photo_score: f32 = 0.0;
    let mut illustration_score: f32 = 0.0;

    // --- Signal 1: EXIF Metadata (Definitive for photography) ---
    if let Some(data) = raw_data {
        let mut cursor = std::io::Cursor::new(data);
        if let Ok(reader) = Reader::new().read_from_container(&mut cursor) {
            let camera_tags = [Tag::Make, Tag::Model, Tag::Software, Tag::DateTime];
            let has_camera_data = reader.fields().any(|f| camera_tags.contains(&f.tag));
            if has_camera_data {
                info!("Style: EXIF camera metadata detected (+5.0 Photo)");
                photo_score += 5.0;
            }
        }
    }

    // --- Signal 2: Alpha Channel / Transparency ---
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let has_transparency = rgba.pixels().step_by(10).take(1000).any(|p| p.0[3] < 255);
        if has_transparency {
            info!("Style: Transparency detected (+3.0 Illustration)");
            illustration_score += 3.0;
        }
    }

    // --- Signal 3: Shannon Entropy ---
    let gray_img = img.to_luma8();
    let mut counts = [0usize; 256];
    for p in gray_img.pixels().step_by(4) {
        counts[p.0[0] as usize] += 1;
    }
    let total_samples = counts.iter().sum::<usize>() as f32;
    let mut entropy: f32 = 0.0;
    if total_samples > 0.0 {
        for &count in counts.iter() {
            if count > 0 {
                let p = count as f32 / total_samples;
                entropy -= p * p.log2();
            }
        }
    }

    let (w, h) = img.dimensions();
    let sample_step = (w / 100).max(1).min(h / 100).max(1);
    let rgb = img.to_rgb8();

    // --- Signal 4: Fuzzy Flatness (adjacent pixel similarity) ---
    // V6 used exact `==` which fails on JPEG-compressed art. Now uses a color
    // distance threshold. 8.0 tolerates JPEG artifacts in real art without
    // making compressed photo gradients (sky, fog) register as "flat".
    const FLAT_THRESHOLD: f32 = 8.0;
    let mut flat_count = 0u64;
    let mut samples = 0u64;
    for y in (0..h.saturating_sub(1)).step_by(sample_step as usize) {
        for x in (0..w.saturating_sub(1)).step_by(sample_step as usize) {
            if color_distance(rgb.get_pixel(x, y), rgb.get_pixel(x + 1, y)) < FLAT_THRESHOLD {
                flat_count += 1;
            }
            samples += 1;
        }
    }
    let flatness = if samples > 0 { flat_count as f32 / samples as f32 } else { 0.0 };

    // --- Signal 5: Quantized Color Palette ---
    // Instead of counting raw unique RGB triples (inflated by JPEG noise), quantize
    // to 4-bit per channel (16×16×16 = 4096 bins). Illustrations cluster into far
    // fewer quantized bins than photographs.
    let mut quant_colors = HashSet::with_capacity(512);
    let mut raw_colors = HashSet::with_capacity(1000);
    for y in (0..h).step_by((sample_step * 2) as usize) {
        for x in (0..w).step_by((sample_step * 2) as usize) {
            let px = rgb.get_pixel(x, y);
            raw_colors.insert(*px);
            // Quantize to 4-bit per channel
            let qr = px.0[0] >> 4;
            let qg = px.0[1] >> 4;
            let qb = px.0[2] >> 4;
            quant_colors.insert((qr, qg, qb));
            if raw_colors.len() > 5000 { break; }
        }
    }
    let color_count = raw_colors.len();
    let quantized_color_count = quant_colors.len();

    // --- Signal 6: Fuzzy Pixel Art Grid Detection ---
    // V6 used exact `==` for grid matching which always fails on JPEG pixel art.
    // Now uses color distance threshold to tolerate JPEG compression artifacts.
    const GRID_THRESHOLD: f32 = 20.0;
    let mut grid_matches = 0u64;
    let mut grid_samples = 0u64;
    let grid_step = sample_step.max(2) as usize;
    for y in (0..h.saturating_sub(2)).step_by(grid_step) {
        for x in (0..w.saturating_sub(2)).step_by(grid_step) {
            let p1 = rgb.get_pixel(x, y);
            let p2 = rgb.get_pixel(x + 2, y);
            if color_distance(p1, p2) < GRID_THRESHOLD {
                grid_matches += 1;
            }
            grid_samples += 1;
        }
    }
    let grid_uniformity = if grid_samples > 0 { grid_matches as f32 / grid_samples as f32 } else { 0.0 };

    // --- Signal 7: Gradient Magnitude Distribution ---
    // Photos have smooth, continuous gradient distributions. Illustrations have a
    // bimodal distribution: many pixels with ~0 gradient (flat fills) and spikes at
    // high gradients (hard edges). We measure this as the ratio of "near-zero gradient"
    // pixels to total — a high ratio means lots of flat fills (illustration signal).
    let mut grad_zero_count = 0u64;
    let mut grad_high_count = 0u64;
    let mut grad_samples = 0u64;
    for y in (1..h.saturating_sub(1)).step_by(sample_step as usize) {
        for x in (1..w.saturating_sub(1)).step_by(sample_step as usize) {
            let center = gray_img.get_pixel(x, y).0[0] as f32;
            let right  = gray_img.get_pixel(x + 1, y).0[0] as f32;
            let below  = gray_img.get_pixel(x, y + 1).0[0] as f32;
            let grad_mag = ((right - center).powi(2) + (below - center).powi(2)).sqrt();
            
            if grad_mag < 3.0 {
                grad_zero_count += 1;
            } else if grad_mag > 40.0 {
                grad_high_count += 1;
            }
            grad_samples += 1;
        }
    }
    let grad_flat_ratio = if grad_samples > 0 { grad_zero_count as f32 / grad_samples as f32 } else { 0.0 };
    let grad_edge_ratio = if grad_samples > 0 { grad_high_count as f32 / grad_samples as f32 } else { 0.0 };

    // --- Signal 8: Saturation Variance ---
    // Photos have naturally varying saturation across the image. Illustrations tend to
    // use uniform, deliberately-chosen saturations within their flat color regions.
    // Low saturation variance = illustration signal.
    let mut sat_sum = 0.0f64;
    let mut sat_sq_sum = 0.0f64;
    let mut sat_samples = 0u64;
    for y in (0..h).step_by((sample_step * 2) as usize) {
        for x in (0..w).step_by((sample_step * 2) as usize) {
            let px = rgb.get_pixel(x, y);
            let r = px.0[0] as f32 / 255.0;
            let g = px.0[1] as f32 / 255.0;
            let b = px.0[2] as f32 / 255.0;
            let max_c = r.max(g).max(b);
            let min_c = r.min(g).min(b);
            let sat = if max_c > 0.0 { (max_c - min_c) / max_c } else { 0.0 };
            sat_sum += sat as f64;
            sat_sq_sum += (sat * sat) as f64;
            sat_samples += 1;
        }
    }
    let sat_mean = if sat_samples > 0 { sat_sum / sat_samples as f64 } else { 0.0 };
    let sat_variance = if sat_samples > 1 {
        (sat_sq_sum / sat_samples as f64 - sat_mean * sat_mean).max(0.0)
    } else { 0.0 };

    // --- Signal 9: NSFW model drawing score (free — already computed if available) ---
    // We can leverage the NSFW model's "drawings" classification as a side signal.
    // Skipped if model isn't loaded (it's not critical).
    let mut nsfw_drawing_score: f32 = 0.0;
    if let Some(model) = NSFW_MODEL.get() {
        let rgba = img.to_rgba8();
        if let Ok(result) = examine(model, &rgba) {
            for class in &result {
                let name_lower = format!("{:?}", class.metric).to_lowercase();
                if name_lower == "drawing" || name_lower == "drawings" {
                    nsfw_drawing_score = class.score;
                }
            }
        }
    }

    // ===================================================================
    // Scoring Logic V7.1 — cross-validated signals to prevent false positives
    // ===================================================================
    
    // --- Entropy signals ---
    if entropy > 7.2 {
        photo_score += 2.0;
    }

    // Low entropy + high flatness = classic illustration (flat fills)
    if entropy < 5.5 && flatness > 0.50 {
        illustration_score += 3.0;
    } else if entropy < 6.0 && flatness > 0.60 {
        illustration_score += 2.0;
    }

    // --- Flatness signals (thresholds raised because fuzzy matching inflates scores) ---
    // With fuzzy matching (threshold=8), even photo gradients score ~0.3-0.5 flatness.
    // Real illustrations with flat fills score 0.6+. Require higher bar.
    if flatness > 0.60 && entropy < 6.5 {
        illustration_score += 2.5;
    } else if flatness > 0.60 && entropy >= 6.5 {
        // High flatness + high entropy = compressed photo with gradients, NOT illustration
        photo_score += 1.0;
    }
    
    if flatness < 0.15 {
        photo_score += 1.5;
    }

    // --- Pixel art detection (now with fuzzy grid matching) ---
    if grid_uniformity > 0.50 && flatness > 0.20 && entropy < 7.0 {
        info!("Style: Pixel art grid pattern detected (uniformity={:.2})", grid_uniformity);
        illustration_score += 3.0;
    }

    // --- Gradient bimodality (cross-validated against entropy) ---
    // Photos with big skies/dark areas have high grad_flat_ratio too, but they
    // also have high entropy from sensor noise. Real illustrations have flat
    // gradients AND low entropy. Only fire this signal when entropy corroborates.
    if grad_flat_ratio > 0.65 && grad_edge_ratio > 0.03 && entropy < 7.0 {
        info!("Style: Gradient bimodality detected (flat={:.2}, edge={:.2}, entropy={:.2}) — illustration signal", 
            grad_flat_ratio, grad_edge_ratio, entropy);
        illustration_score += 3.0;
    } else if grad_flat_ratio > 0.55 && grad_edge_ratio > 0.02 && entropy < 6.5 {
        illustration_score += 1.5;
    } else if grad_flat_ratio < 0.30 {
        // Very few flat gradient regions = continuous photographic texture
        photo_score += 1.5;
    }

    // --- Quantized palette clustering (cross-validated against entropy) ---
    // Photos with muted/monochrome palettes can have low quantized colors too,
    // so require entropy corroboration.
    if quantized_color_count < 80 && entropy < 6.5 {
        info!("Style: Low quantized palette ({} bins, entropy={:.2}) — strong illustration signal", quantized_color_count, entropy);
        illustration_score += 3.0;
    } else if quantized_color_count < 200 && entropy < 6.5 {
        illustration_score += 1.5;
    } else if quantized_color_count > 800 {
        photo_score += 1.5;
    }

    // --- Saturation variance (cross-validated against entropy) ---
    // Low sat_variance in photos happens with monochrome/muted shots (fog, B&W).
    // Only count as illustration signal if entropy also supports it.
    if sat_variance < 0.008 && sat_mean > 0.05 && entropy < 6.5 {
        info!("Style: Low saturation variance ({:.4}, entropy={:.2}) — illustration signal", sat_variance, entropy);
        illustration_score += 2.0;
    } else if sat_variance > 0.04 {
        photo_score += 1.0;
    }

    // --- Legacy color + entropy combos ---
    if color_count > 4000 && entropy > 7.0 {
        photo_score += 2.0;
    }
    if color_count < 1000 && flatness > 0.15 {
        illustration_score += 2.0;
    }

    // --- NSFW model drawing classifier as tiebreaker ---
    if nsfw_drawing_score > 0.70 {
        illustration_score += 1.5;
    } else if nsfw_drawing_score > 0.40 {
        illustration_score += 0.5;
    }

    info!("Ensemble V7.1 — Entropy: {:.2}, Flat: {:.2}, Colors: {} (quant: {}), Grid: {:.2}, GradFlat: {:.2}, GradEdge: {:.2}, SatVar: {:.4}, DrawingML: {:.2}", 
        entropy, flatness, color_count, quantized_color_count, grid_uniformity, grad_flat_ratio, grad_edge_ratio, sat_variance, nsfw_drawing_score);
    info!("Total Scores — Photo: {:.1}, Illustration: {:.1}", photo_score, illustration_score);

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

    // 5. Encode to Base64 (for Gemini request)
    let mut buffer = Cursor::new(Vec::new());
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 95);
    processed_img.write_with_encoder(jpeg_encoder)?;
    
    let base64_data = general_purpose::STANDARD.encode(buffer.into_inner());

    Ok(ProcessedImage {
        base64_data,
        ratio_name: nearest.name.to_string(),
    })
}

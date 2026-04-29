use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::env;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Deserialize, Debug)]
pub struct ReplicatePredictionResponse {
    pub id: String,
    pub status: String,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct ReplicateClient {
    client: Client,
    token: String,
}

impl ReplicateClient {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let token = env::var("REPLICATE_API_TOKEN").map_err(|_| "REPLICATE_API_TOKEN not set")?;
        
        // Single persistent client for high-throughput polling and reliable POSTs
        let client = Client::builder()
            .http1_only() // CRITICAL: Avoid HTTP/2 stream hangs on certain VPS environments
            .user_agent("Upscaler-Backend/1.1 (High-Throughput)")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(300))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            client,
            token,
        })
    }

    pub async fn run_blip_caption(&self, image_url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "image": image_url
            }
        });
        info!("Running fast BLIP-base captioning pass...");
        let start = std::time::Instant::now();
        let res = match self.run_replicate_model(
            "salesforce/blip",
            "2e1dddc8621f72155f24cf2e0adbde548458d3cab9f00c0139eea840d0ac4746",
            req_body
        ).await {
            Ok(c) => {
                info!("Fast BLIP generated caption in {}ms", start.elapsed().as_millis());
                c
            },
            Err(e) => {
                tracing::error!("BLIP captioning failed: {}", e);
                return Err(e);
            }
        };

        // The response is usually a string starting with "Caption: ..." or just the caption
        let caption = res.replace("Caption:", "").trim().to_string();
        info!("BLIP-3 generated caption: {}", caption);
        Ok(caption)
    }

    pub async fn run_seesr(
        &self,
        image_url: &str,
        caption: Option<String>,
        quality: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut prompt = String::new();
        prompt.push_str("high quality, detailed, sharp, photographic, masterwork, 8k resolution, highly detailed textures, natural skin, realistic fur.");
        if let Some(cap) = caption {
            prompt.push_str(&format!(" {}.", cap));
        }

        // Adaptive upscale factor
        let upscale = match quality {
            "4x" | "4K" => 4,
            _ => 2,
        };

        let input = serde_json::json!({
            "image": image_url,
            "user_prompt": prompt, 
            "negative_prompt": "AI-generated look, plastic, airbrushed, cartoon, low resolution, blurry, dotted, noise, smooth, unnatural textures",
            "num_inference_steps": 30, // Reduced from 50 to 30 for cost efficiency and speed
            "cfg_scale": 5.5, 
            "scale_factor": upscale,
        });

        info!("Standard Mode V6.0: Running SeeSR (cswry) [Upscale: {}x, Steps: 30]", upscale);
        let version = "989cf3a66fd209363de347c3129d95d9fe639e44533ab47e07a6dfb3f250b6e3";
        self.run_replicate_model("cswry/seesr", version, serde_json::json!({ "input": input })).await
    }

    pub async fn run_p_image_edit(
        &self,
        image_url: &str,
        caption: Option<String>,
        settings: &crate::prompts::PromptSettings,
        _is_low_res: bool,
        _is_grayscale: bool,
        _is_premium_pre_pass: bool,
        style: crate::processor::ImageStyle,
        input_mp: f32,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let creativity = settings.creativity;
        let _refinement = settings.refinement;
        
        let mut prompt = String::new();
        
        // 1. Mandatory Trigger Word
        prompt.push_str("tok_enhance. ");

        // 2. Identity & Fidelity Anchor
        prompt.push_str("Subtle photographic cleanup of image 1. ");
        prompt.push_str("Strictly preserve the original identity, composition, and soul. ");

        let (effective_style, category) = self.decide_style_and_category(caption.as_deref(), style);
        
        // 3. Resolution-Specific Strategy (Reconstruction vs. Subtle Cleanup)
        if input_mp < 0.15 {
            // --- BRANCH A: RECONSTRUCTION (Ultra Low-Res, e.g., < 300px) ---
            prompt.push_str("Structural restoration pass. Remove heavy blur and fix pixelation. ");
            prompt.push_str("Cleanly reconstruct missing details based on image 1. ");
            if let Some(cap) = caption.as_ref() {
                prompt.push_str(&format!("Accurately restore the features of {}. ", cap));
            }
        } else {
            // --- BRANCH B: CONSERVATIVE CLEANUP (Standard/High-Res) ---
            prompt.push_str("High-fidelity artifact removal. Remove JPEG compression, noise, and digital grain. ");
            prompt.push_str("Maintain 100% fidelity to the original features. Do not regenerate textures. ");
            prompt.push_str("Strictly no changes to the subject or background. ");
        }

        // 4. Texture & Detail Preservation
        if effective_style == crate::processor::ImageStyle::Photography {
            if input_mp < 0.15 {
                match category.as_str() {
                    "Portrait" => prompt.push_str("Maintain skin texture and sharp iris detail. "),
                    "Wildlife" => prompt.push_str("Maintain natural fur flow and sharp eye detail. "),
                    _ => prompt.push_str("Maintain realistic photographic micro-textures. "),
                }
            } else {
                prompt.push_str("Preserve existing photographic textures. Maintain organic softness. ");
            }
        } else {
            prompt.push_str("Maintain clean line art and original color fields. ");
        }

        // 5. Lighting & Finish
        prompt.push_str("Balanced exposure, soft natural lighting. ");
        prompt.push_str("Strictly preserve smooth out-of-focus bokeh background. Do not sharpen or add detail to the background. ");
        prompt.push_str("No halos, no white outlines, no over-sharpening. ");

        // 6. Creativity Scaling (Affects Prompt Density)
        if creativity > 0.7 && input_mp < 0.3 {
            prompt.push_str("Enhanced reconstruction. ");
        } else if creativity < 0.3 {
            prompt.push_str("Minimal cleanup only. ");
        }

        // 7. Overhauled Negative Prompt (Blocking the "AI Look")
        let neg_prompt = "AI look, digital painting, generative artifacts, plastic skin, airbrushed, waxiness, smeared details, over-sharpened, etched textures, cinematic lighting, dramatic shadows, color shift, cartoonish, digital art look, beauty filter, fake textures, distorted anatomy, artificial digital noise, high contrast, crushed blacks, halos, white outlines, over-etched edges, artificial sharpness, over-saturated, blurry, pixelated, jpeg artifacts, sharpening artifacts in bokeh, textured blur".to_string();

        // 8. Adaptive LoRA Scaling (Significantly Lowered for Realism)
        let base_scale = if input_mp < 0.15 { 0.70 } else if input_mp < 1.0 { 0.35 } else { 0.20 };
        let lora_scale = (base_scale + (creativity - 0.5) * 0.3).clamp(0.1, 1.0);

        let mut input = serde_json::json!({
            "images": [image_url],
            "prompt": prompt,
            "negative_prompt": neg_prompt,
            "turbo": false,
            "aspect_ratio": "match_input_image",
            "lora_weights": "https://huggingface.co/davidberenstein1957/p-image-edit-photo-enhancement-lora/resolve/main/weights.safetensors",
            "lora_scale": lora_scale
        });

        if let Some(seed) = settings.seed {
            input["seed"] = serde_json::json!(seed);
        }

        let req_body = serde_json::json!({
            "input": input
        });

        info!("V3 Pipeline — Branching: [MP: {:.2}, Scale: {:.2}], Style: {:?}, Category: {}", input_mp, lora_scale, effective_style, category);
        self.run_replicate_model(
            "prunaai/p-image-edit-lora",
            "191152bf662a44024fe326e61595d4f84c0293afdee7ff08d973d5e399973a4e",
            req_body
        ).await
    }

    pub async fn run_p_image_upscale(
        &self,
        image_url: &str,
        quality: &str,
        creativity: f32,
        input_mp: f32, // New parameter
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Map quality to target MP
        let target_mp = match quality {
            "2x" | "2K" => 4,
            "4x" | "4K" => 8,
            "6x" | "6K" => 8, // Cap at 8MP for p-image-upscale
            _ => 4,
        };

        // Resolution-aware tuning for polish
        let enhance_details = if input_mp < 0.3 { true } else { creativity >= 0.4 };
        let enhance_realism = if input_mp < 0.3 { creativity >= 0.3 } else { creativity >= 0.6 };

        let input = serde_json::json!({
            "image": image_url,
            "target": target_mp,
            "upscale_mode": "target",
            "enhance_details": enhance_details,
            "enhance_realism": enhance_realism,
            "output_format": "jpg",
            "output_quality": 95
        });

        let req_body = serde_json::json!({
            "input": input
        });

        info!("Running Pruna AI P-Image-Upscale — [Target: {}MP, Details: {}, Realism: {}]", target_mp, enhance_details, enhance_realism);
        self.run_replicate_model(
            "prunaai/p-image-upscale",
            "9018fe338f75cea08d1e3abc5f4f795d62594abf94326d5e590090f593bb1bac",
            req_body
        ).await
    }

    pub async fn run_real_esrgan(&self, image_url: &str, quality: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let scale = match quality {
            "2x" | "2K" => 2,
            "4x" | "4K" => 4,
            "6x" | "6K" => 4,
            _ => 2,
        };

        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "upscale": scale,
                "face_enhance": false,
            }
        });

        info!("Running Real-ESRGAN Upscale (Factor: {}x)...", scale);
        // Using a stable version of nightmareai/real-esrgan
        self.run_replicate_model(
            "nightmareai/real-esrgan",
            "42fed1c4974146d4d2414e2be2c5277c7fcf05fcc3a73abf41610695738c1d7b",
            req_body
        ).await
    }

    pub async fn run_real_esrgan_2x(&self, image_url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "scale": 2,
                "face_enhance": false
            }
        });

        info!("Running Real-ESRGAN 2x Technical Restoration...");
        self.run_replicate_model(
            "nightmareai/real-esrgan",
            "42fed1c4974146d4d2414e2be2c5277c7fcf05fcc3a73abf41610695738c1d7b",
            req_body
        ).await
    }

    pub async fn run_topaz(&self, image_url: &str, upscale_factor: &str, style: &str, topaz_mode: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let enhance_model = match topaz_mode {
            "Low Quality Recovery" => "Low Resolution V2",
            "Standard" => "Standard V2",
            "High Fidelity" => "High Fidelity V2",
            "CGI" => "CGI",
            "Text Refine" => "Text Refine",
            _ => {
                if style == "PHOTOGRAPHY" { "Standard V2" } else { "CGI" }
            }
        };
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "enhance_model": enhance_model,
                "upscale_factor": upscale_factor,
                "face_enhancement": false,
                "subject_detection": "None",
            }
        });

        info!("Starting Replicate Topaz job...");
        self.run_replicate_model("topazlabs/image-upscale", "2fdc3b86a01d338ae89ad58e5d9241398a8a01de9b0dda41ba8a0434c8a00dc3", req_body).await
    }

    pub async fn run_replicate_model(&self, model: &str, version: &str, req_body: serde_json::Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempts = 0;
        let mut resp;
        
        loop {
            attempts += 1;
            info!("Replicate [{}]: Sending request (Attempt {})...", model, attempts);
            
            let req = self.client.post("https://api.replicate.com/v1/predictions")
                .bearer_auth(&self.token)
                .json(&serde_json::json!({
                    "version": version,
                    "input": req_body["input"]
                }));

            // Use explicit tokio timeout to prevent silent hangs in the reqwest state machine
            let send_future = req.send();
            resp = match tokio::time::timeout(Duration::from_secs(45), send_future).await {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => {
                    warn!("Replicate [{}]: Connection error (Attempt {}): {}. Retrying...", model, attempts, e);
                    if attempts >= 5 { return Err(e.into()); }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                Err(_) => {
                    warn!("Replicate [{}]: Request timed out at 45s (Attempt {}). Connection likely stuck. Retrying...", model, attempts);
                    if attempts >= 3 { return Err("Replicate POST request timed out persistently".into()); }
                    continue;
                }
            };

            if resp.status().is_success() {
                info!("Replicate [{}]: Request accepted. Status: {}", model, resp.status());
                break;
            }

            let status = resp.status();
            if status == 429 && attempts < 10 {
                let error_body = resp.text().await.unwrap_or_default();
                let wait_secs = serde_json::from_str::<serde_json::Value>(&error_body)
                    .ok()
                    .and_then(|v| v.get("retry_after").and_then(|ra| ra.as_u64()))
                    .unwrap_or(attempts as u64 * 3 + 2);

                warn!("Replicate Throttled (429). Waiting {}s before retry {}/10...", wait_secs, attempts);
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                continue;
            } else {
                let txt = resp.text().await?;
                return Err(format!("Replicate API error ({}): {}", status, txt).into());
            }
        }

        let mut pred: ReplicatePredictionResponse = resp.json().await?;
        let start_poll = std::time::Instant::now();
        
        while pred.status == "starting" || pred.status == "processing" {
            if start_poll.elapsed().as_secs() > 600 {
                return Err("Replicate job timed out after 10 minutes".into());
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
            
            let poll_req = self.client.get(&format!("https://api.replicate.com/v1/predictions/{}", pred.id))
                .bearer_auth(&self.token);

            let poll_res = match tokio::time::timeout(Duration::from_secs(20), poll_req.send()).await {
                Ok(Ok(r)) => r,
                _ => {
                    warn!("Replicate [{}]: Polling request hung or timed out. Retrying poll...", model);
                    continue;
                }
            };
                
            if !poll_res.status().is_success() {
                 warn!("Replicate [{}]: Polling failed (status: {}). Retrying poll...", model, poll_res.status());
                 continue;
            }
            pred = poll_res.json().await?;
            info!("Replicate [{}]: Status: {} ({}s)", model, pred.status, start_poll.elapsed().as_secs());
        }

        if pred.status == "failed" || pred.status == "canceled" {
            return Err(format!("Replicate job failed: {:?}", pred.error).into());
        }

        if let Some(out) = pred.output {
            if let Some(out_url) = out.as_str() {
                return Ok(out_url.to_string());
            } else if let Some(out_arr) = out.as_array() {
                let joined: String = out_arr.iter().filter_map(|v| v.as_str()).collect();
                if !joined.is_empty() {
                    return Ok(joined);
                }
            }
            Err("No output from Replicate".into())
        } else {
            Err("No output from Replicate".into())
        }
    }

    pub fn decide_style_and_category(&self, caption: Option<&str>, local_style: crate::processor::ImageStyle) -> (crate::processor::ImageStyle, String) {
        let low_caps = caption.unwrap_or_default().to_lowercase();
        
        // 1. Determine Category (Subject Matter)
        let category = self.derive_category(caption, local_style);

        // 2. Determine Style (Photography vs Illustration)
        // PRIORITY A: Explicit keywords in caption
        let style = if low_caps.contains("illustration") || low_caps.contains("drawing") || 
                       low_caps.contains("anime") || low_caps.contains("sketch") || 
                       low_caps.contains("painting") || low_caps.contains("vector") ||
                       low_caps.contains("digital art") || low_caps.contains("cartoon") ||
                       low_caps.contains("comic") || low_caps.contains("cgi") ||
                       low_caps.contains("3d render") || low_caps.contains("pixel art") {
            crate::processor::ImageStyle::Illustration
        } else if low_caps.contains("photograph") || low_caps.contains("realistic") || 
                  low_caps.contains("photo") || low_caps.contains("snapshot") || 
                  low_caps.contains("cinematic") || low_caps.contains("35mm") ||
                  low_caps.contains("raw photo") || low_caps.contains("portrait") ||
                  low_caps.contains("wildlife") {
            crate::processor::ImageStyle::Photography
        } else {
            // PRIORITY B: Category Inference
            match category.as_str() {
                "Portrait" | "Wildlife" | "Nature" | "Food" | "Macro" => {
                    // Natural subjects are Photography unless explicitly stated otherwise above
                    crate::processor::ImageStyle::Photography
                },
                "Architecture" | "Product" | "Vehicle" => {
                    // Neutral categories fallback to local style or photography
                    if local_style == crate::processor::ImageStyle::Illustration {
                         crate::processor::ImageStyle::Illustration
                    } else {
                         crate::processor::ImageStyle::Photography
                    }
                },
                _ => {
                    // PRIORITY C: Local Classifier Fallback
                    local_style
                }
            }
        };

        (style, category)
    }

    pub fn derive_category(&self, caption: Option<&str>, style: crate::processor::ImageStyle) -> String {
        let low_caps = caption.unwrap_or_default().to_lowercase();
        
        // Portrait keywords
        if low_caps.contains("face") || low_caps.contains("person") || low_caps.contains("man") || 
           low_caps.contains("woman") || low_caps.contains("human") || low_caps.contains("eye") || 
           low_caps.contains("skin") || low_caps.contains("portrait") || low_caps.contains("girl") || 
           low_caps.contains("boy") || low_caps.contains("selfie") {
            return "Portrait".to_string();
        }

        // Wildlife keywords
        if low_caps.contains("animal") || low_caps.contains("deer") || low_caps.contains("fur") || 
           low_caps.contains("bird") || low_caps.contains("pet") || low_caps.contains("dog") || 
           low_caps.contains("cat") || low_caps.contains("wildlife") || low_caps.contains("horse") || 
           low_caps.contains("lion") || low_caps.contains("tiger") || low_caps.contains("fish") || 
           low_caps.contains("insect") || low_caps.contains("feathers") || low_caps.contains("mammal") {
            return "Wildlife".to_string();
        }

        // Nature/Landscape
        if low_caps.contains("tree") || low_caps.contains("forest") || low_caps.contains("mountain") || 
           low_caps.contains("sky") || low_caps.contains("grass") || low_caps.contains("field") || 
           low_caps.contains("landscape") || low_caps.contains("outdoors") || 
           low_caps.contains("flower") || low_caps.contains("leaf") || low_caps.contains("plant") ||
           low_caps.contains("beach") || low_caps.contains("ocean") || low_caps.contains("river") ||
           low_caps.contains("cloud") || low_caps.contains("sunset") || low_caps.contains("nature") {
            return "Nature".to_string();
        }

        // Architecture
        if low_caps.contains("building") || low_caps.contains("house") || low_caps.contains("street") || 
           low_caps.contains("city") || low_caps.contains("room") || low_caps.contains("interior") ||
           low_caps.contains("architecture") || low_caps.contains("window") || low_caps.contains("door") ||
           low_caps.contains("tower") || low_caps.contains("bridge") || low_caps.contains("temple") {
            return "Architecture".to_string();
        }

        // Vehicle
        if low_caps.contains("car") || low_caps.contains("truck") || low_caps.contains("plane") || 
           low_caps.contains("boat") || low_caps.contains("bike") || low_caps.contains("cycle") || 
           low_caps.contains("motorcycle") || low_caps.contains("ship") || low_caps.contains("aircraft") {
            return "Vehicle".to_string();
        }

        // Product
        if low_caps.contains("product") || low_caps.contains("bottle") || low_caps.contains("watch") || 
           low_caps.contains("jewelry") || low_caps.contains("shoe") || low_caps.contains("gadget") || 
           low_caps.contains("device") || low_caps.contains("electronics") || low_caps.contains("cosmetic") {
            return "Product".to_string();
        }

        // Texture/Material/Macro
        if low_caps.contains("texture") || low_caps.contains("material") || low_caps.contains("pattern") || 
           low_caps.contains("surface") || low_caps.contains("wood") || low_caps.contains("metal") || 
           low_caps.contains("fabric") || low_caps.contains("stone") || low_caps.contains("rock") || 
           low_caps.contains("leather") || low_caps.contains("macro") || low_caps.contains("close up") {
            return "Texture".to_string();
        }

        // Food
        if low_caps.contains("food") || low_caps.contains("drink") || low_caps.contains("fruit") ||
           low_caps.contains("vegetable") || low_caps.contains("meat") || low_caps.contains("plate") ||
           low_caps.contains("dish") || low_caps.contains("meal") {
            return "Food".to_string();
        }

        // Fallbacks
        match style {
            crate::processor::ImageStyle::Photography => "Photography".to_string(),
            crate::processor::ImageStyle::Illustration => "Illustration".to_string(),
        }
    }
}

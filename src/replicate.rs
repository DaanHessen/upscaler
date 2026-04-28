use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::env;
use std::time::Duration;
use tracing::info;

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
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
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

    pub async fn run_p_image_edit(
        &self,
        image_url: &str,
        caption: Option<String>,
        settings: &crate::prompts::PromptSettings,
        is_low_res: bool,
        is_grayscale: bool,
        is_premium_pre_pass: bool,
        style: crate::processor::ImageStyle
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let creativity = settings.creativity;
        let refinement = settings.refinement;
        
        let mut prompt = String::new();
        let mut neg_prompt = "blurry, smooth, plastic, cartoon, noise, artifacts, smeared, fake, distorted, weird textures, artificial patterns".to_string();

        // 1. Core Instruction: FIDELITY & IDENTITY
        prompt.push_str("Modify image 1 with ultra-high-fidelity enhancement. ");
        prompt.push_str("Strictly preserve the original soul, identity, lighting, and composition. ");
        
        // 2. Lighting Anchors (Preventing the 'deep-fried' / high-contrast look)
        prompt.push_str("Well-lit, high-key lighting, balanced exposure, natural raw photo aesthetic, shot on 35mm lens. ");

        if is_grayscale {
            prompt.push_str("Strictly black and white, monochrome, high-contrast grayscale. ");
            neg_prompt.push_str(", color, saturation, sepia, hue, tint");
        }

        let (effective_style, category) = self.decide_style_and_category(caption.as_deref(), style);
        
        info!("Smarter Prompting — Final Verdict: [Style: {:?}, Category: {}], Caption: {:?}", effective_style, category, caption);

        // 3. Resolution-Specific Strategies
        if is_low_res {
            // --- BRANCH A: RECONSTRUCTION (Low-res) ---
            prompt.push_str("Identity-locked reconstruction. Rebuild missing high-frequency data from image 1. Sharp optical clarity");
            if let Some(cap) = caption.as_ref() {
                prompt.push_str(&format!(" of {},", cap));
            }
        } else if is_premium_pre_pass {
            // --- BRANCH B: PRE-PASS (Premium) ---
            prompt.push_str("Micro-detail refinement and pristine image cleanup. Remove artifacts while preserving all original high-frequency details");
            if let Some(cap) = caption.as_ref() {
                prompt.push_str(&format!(" of {},", cap));
            }
        } else {
            // --- BRANCH C: ENHANCEMENT (Standard High-res) ---
            prompt.push_str("Subtle high-fidelity refinement and upscale of existing textures only. No new features");
            if let Some(cap) = caption.as_ref() {
                prompt.push_str(&format!(" of the {}", cap));
            }
        }

        // 4. Texture Locking Vocabulary (Additive Realism)
        if effective_style == crate::processor::ImageStyle::Photography {
            match category.as_str() {
                "Portrait" => prompt.push_str(", implement visible skin pores, individual eyelash definition, natural moisture, and realistic iris texture"),
                "Wildlife" => prompt.push_str(", implement individual hair follicles, realistic fur depth, and sharp organic eye detail"),
                "Nature" | "Landscape" => prompt.push_str(", implement intricate organic detail, crisp foliage textures, and atmospheric depth"),
                "Architecture" => prompt.push_str(", implement sharp geometric precision, clean architectural lines, and realistic stone/metal/glass textures"),
                "Product" => prompt.push_str(", implement pristine product surfaces, sharp labels, and realistic material textures"),
                "Vehicle" => prompt.push_str(", implement sharp mechanical detail, realistic paint reflections, and crisp material textures"),
                "Texture" | "Macro" => prompt.push_str(", implement extreme macro detail, sharp tactile surfaces, and realistic material micro-patterns"),
                _ => prompt.push_str(", implement natural organic micro-textures and realistic clarity"),
            }
        } else {
            // Illustration / Digital Art
            match category.as_str() {
                "Architecture" | "Geometric" => prompt.push_str(", maintain sharp geometric precision and clean sharp edges"),
                _ => prompt.push_str(", maintain clean line art, smooth vector fills, and pristine original detail"),
            }
        }

        // 5. Creativity-Based Intensity Scaling
        if creativity < 0.3 {
            prompt.push_str(". Minimal changes, subtle enhancement, strictly preserve every pixel.");
        } else if creativity > 0.7 {
            prompt.push_str(". Generative reconstruction, highly detailed texture injection, enhanced realism.");
        }

        // 6. Refinement (Edge Sharpening)
        if refinement {
            prompt.push_str(". Crisp edge definition, sharp high-frequency micro-details.");
        } else {
            prompt.push_str(". Natural smooth textures, soft organic finish, realistic preservation.");
        }

        // Final Anchor
        prompt.push_str(" Maintain original dynamic range. Do not crush blacks or blow out highlights. ");
        prompt.push_str("Strictly do not add new features, do not change anatomy, and do not alter the original color palette.");

        // Overhauled Negative Prompt
        neg_prompt = "plastic skin, airbrushed, waxiness, smeared details, over-sharpened, etched textures, cinematic lighting, dramatic shadows, color shift, cartoonish, digital art look, beauty filter, fake textures, distorted anatomy, artificial digital noise, high contrast, crushed blacks, oversaturated, neon, artificial fur, painting look, smeared details, weird anatomy, extra limbs, fused fingers, low quality, blurry, pixelated, jpeg artifacts".to_string();


        let mut input = serde_json::json!({
            "images": [image_url],
            "prompt": prompt,
            "negative_prompt": neg_prompt,
            "turbo": false,
            "aspect_ratio": "match_input_image",
            "replicate_weights": if is_low_res { "default" } else { "light_restoration" }
        });

        if let Some(seed) = settings.seed {
            input["seed"] = serde_json::json!(seed);
        }

        let req_body = serde_json::json!({
            "input": input
        });

        info!("Running Pruna AI P-Image-Edit (Restoration Pass) with creativity={} and refinement={}...", creativity, refinement);
        self.run_replicate_model(
            "prunaai/p-image-edit",
            "5bf99c2386ca54e33758b7b4d360cf2b9e0f2b61966cd764363173ab3810935b",
            req_body
        ).await
    }

    pub async fn run_p_image_upscale(
        &self,
        image_url: &str,
        quality: &str,
        creativity: f32,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Map quality to target MP
        let target_mp = match quality {
            "2x" | "2K" => 4,
            "4x" | "4K" => 8,
            "6x" | "6K" => 8, // Cap at 8MP for p-image-upscale
            _ => 4,
        };

        let input = serde_json::json!({
            "image": image_url,
            "target": target_mp,
            "upscale_mode": "target",
            "enhance_details": creativity >= 0.4,
            "enhance_realism": creativity >= 0.7,
            "output_format": "jpg",
            "output_quality": 95
        });

        let req_body = serde_json::json!({
            "input": input
        });

        info!("Running Pruna AI P-Image-Upscale (Final Polish) to {}MP...", target_mp);
        self.run_replicate_model(
            "prunaai/p-image-upscale",
            "9018fe338f75cea08d1e3abc5f4f795d62594abf94326d5e590090f593bb1bac",
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

    async fn run_replicate_model(&self, model: &str, version: &str, req_body: serde_json::Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempts = 0;
        let mut resp;
        loop {
            resp = self.client.post("https://api.replicate.com/v1/predictions")
                .bearer_auth(&self.token)
                .json(&serde_json::json!({
                    "version": version,
                    "input": req_body["input"]
                }))
                .send()
                .await?;

            if resp.status().is_success() {
                break;
            }

            let status = resp.status();
            if status == 429 && attempts < 10 {
                attempts += 1;
                
                let retry_header = resp.headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                let error_body = resp.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
                
                // Try to get wait time from JSON body first (Replicate often puts it there)
                let body_retry_after = serde_json::from_str::<serde_json::Value>(&error_body)
                    .ok()
                    .and_then(|v| v.get("retry_after").and_then(|ra| ra.as_u64()))
                    .or(retry_header);

                let wait_secs = body_retry_after.unwrap_or_else(|| 2_u64.pow(attempts));
                info!("Replicate API throttled (429) for {}. Reason: {}. Retrying in {} seconds...", model, error_body, wait_secs);
                
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            } else {
                let txt = resp.text().await?;
                return Err(format!("Replicate API error ({}): {}", status, txt).into());
            }
        }

        let mut pred: ReplicatePredictionResponse = resp.json().await?;
        
        while pred.status == "starting" || pred.status == "processing" {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let get_resp = self.client.get(&format!("https://api.replicate.com/v1/predictions/{}", pred.id))
                .bearer_auth(&self.token)
                .send()
                .await?;
            if !get_resp.status().is_success() {
                 return Err("Failed to poll Replicate".into());
            }
            pred = get_resp.json().await?;
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

    async fn run_replicate_model_by_name(&self, model_owner: &str, model_name: &str, req_body: serde_json::Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempts = 0;
        let mut resp;
        loop {
            resp = self.client.post(&format!("https://api.replicate.com/v1/models/{}/{}/predictions", model_owner, model_name))
                .bearer_auth(&self.token)
                .json(&serde_json::json!({
                    "input": req_body["input"]
                }))
                .send()
                .await?;

            if resp.status().is_success() {
                break;
            }

            let status = resp.status();
            if status == 429 && attempts < 10 {
                attempts += 1;
                
                let retry_header = resp.headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                let error_body = resp.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
                
                let body_retry_after = serde_json::from_str::<serde_json::Value>(&error_body)
                    .ok()
                    .and_then(|v| v.get("retry_after").and_then(|ra| ra.as_u64()))
                    .or(retry_header);

                let wait_secs = body_retry_after.unwrap_or_else(|| 2_u64.pow(attempts));
                info!("Replicate API throttled (429) for {}/{}. Reason: {}. Retrying in {} seconds...", model_owner, model_name, error_body, wait_secs);
                
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            } else {
                let txt = resp.text().await?;
                return Err(format!("Replicate API error ({}): {}", status, txt).into());
            }
        }

        let mut pred: ReplicatePredictionResponse = resp.json().await?;
        
        while pred.status == "starting" || pred.status == "processing" {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let get_resp = self.client.get(&format!("https://api.replicate.com/v1/predictions/{}", pred.id))
                .bearer_auth(&self.token)
                .send()
                .await?;
            if !get_resp.status().is_success() {
                 return Err("Failed to poll Replicate".into());
            }
            pred = get_resp.json().await?;
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

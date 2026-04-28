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
                "image": image_url,
                "prompt": "Describe this image in a short, comma-separated list of keywords and main subjects."
            }
        });
        info!("Running BLIP-3 captioning pass...");
        let res = self.run_replicate_model_by_name(
            "zsxkib",
            "blip-3",
            req_body
        ).await?;

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
        is_premium_pre_pass: bool
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let creativity = settings.creativity;
        let refinement = settings.refinement;
        
        let mut prompt = String::new();
        let mut neg_prompt = "blurry, smooth, plastic, cartoon, noise, artifacts, smeared, fake, distorted, weird textures, artificial patterns".to_string();

        // 1. Core Instruction: FIDELITY FIRST
        prompt.push_str("Maintain absolute fidelity to the original image's soul, composition, and lighting. ");
        prompt.push_str("Strictly preserve the original color balance and white balance. No color shifting. ");

        if is_grayscale {
            prompt.push_str("Strictly black and white, monochrome, high-contrast grayscale. ");
            neg_prompt.push_str(", color, saturation, sepia, hue, tint");
        }

        let is_human = caption.as_ref().map(|c| {
            let c = c.to_lowercase();
            c.contains("skin") || c.contains("person") || c.contains("human") || 
            c.contains("face") || c.contains("foot") || c.contains("arm") || 
            c.contains("hand") || c.contains("man") || c.contains("woman")
        }).unwrap_or(false);

        if is_low_res {
            // --- BRANCH A: RECONSTRUCTION (Low-res) ---
            prompt.push_str("Clean high-fidelity reconstruction and detail restoration");
            if let Some(cap) = caption {
                prompt.push_str(&format!(" of {},", cap));
            }
            if is_human {
                prompt.push_str(" preserve soft natural skin textures and organic smooth gradients, realistic human appearance");
            } else {
                prompt.push_str(" crisp optical clarity, natural organic textures");
            }
        } else if is_premium_pre_pass {
            // --- BRANCH B: PRE-PASS (Premium) ---
            prompt.push_str("Gentle artifact removal, pristine image cleanup, noise reduction");
            if let Some(cap) = caption {
                prompt.push_str(&format!(" of {},", cap));
            }
            prompt.push_str(" remove jpeg artifacts, preserve original tonal balance");
        } else {
            // --- BRANCH C: ENHANCEMENT (Standard High-res) ---
            prompt.push_str("Professional high-fidelity enhancement, clean optical clarity");
            if let Some(cap) = caption {
                prompt.push_str(&format!(" of the {}", cap));
            }
            if is_human {
                prompt.push_str(", preserve natural skin softness and realistic human detail");
            } else {
                prompt.push_str(", professional studio finish, natural textures");
            }
        }

        // Apply Edge Sharpening preference
        if refinement {
            prompt.push_str(", crisp edge definition, sharp high-frequency details");
        } else {
            prompt.push_str(", natural smooth textures, soft organic finish, realistic preservation");
        }

        // Final Anchor
        prompt.push_str(". Do not add new features, do not change the original anatomy, and do not alter the original color grading.");

        neg_prompt.push_str(", color shift, color grading, stylized, over-sharpened, etched, scaly, non-human skin, uncanny valley, artificial texture");

        let mut input = serde_json::json!({
            "images": [image_url],
            "prompt": prompt,
            "negative_prompt": neg_prompt,
            "turbo": false,
            "aspect_ratio": "match_input_image"
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
}

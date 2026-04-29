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
        
        let client = Client::builder()
            .http1_only()
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

        let caption = res.replace("Caption:", "").trim().to_string();
        info!("BLIP generated caption: {}", caption);
        Ok(caption)
    }

    pub async fn run_restore_image(
        &self,
        image_url: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "input_image": image_url,
                "output_format": "png",
                "safety_tolerance": 2
            }
        });
        info!("Running AI Restoration pass (flux-kontext-apps/restore-image)...");
        self.run_replicate_model(
            "flux-kontext-apps/restore-image",
            "", // Empty version triggers the model-specific latest endpoint
            req_body
        ).await
    }

    pub async fn run_topaz(
        &self,
        image_url: &str,
        upscale_factor: &str,
        enhance_model: &str,
        face_enhancement: bool,
        noise_reduction: i32,
        sharpen: i32,
        remove_artifacts: i32,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "enhance_model": enhance_model,
                "upscale_factor": upscale_factor,
                "output_format": "jpg",
                "subject_detection": "All",
                "face_enhancement": face_enhancement,
                "noise_reduction": noise_reduction,
                "sharpen": sharpen,
                "remove_artifacts": remove_artifacts
            }
        });

        info!("Running Topaz Labs [{}] [Upscale: {}, Face: {}, Noise: {}, Sharpen: {}]", 
            enhance_model, upscale_factor, face_enhancement, noise_reduction, sharpen);
        self.run_replicate_model(
            "topazlabs/image-upscale",
            "2fdc3b86a01d338ae89ad58e5d9241398a8a01de9b0dda41ba8a0434c8a00dc3",
            req_body
        ).await
    }

    pub async fn run_replicate_model(&self, model: &str, version: &str, req_body: serde_json::Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempts = 0;
        let mut resp;
        
        let url = if version.is_empty() {
            format!("https://api.replicate.com/v1/models/{}/predictions", model)
        } else {
            "https://api.replicate.com/v1/predictions".to_string()
        };

        loop {
            attempts += 1;
            info!("Replicate [{}]: Sending request (Attempt {})...", model, attempts);
            
            let mut body = serde_json::json!({
                "input": req_body["input"]
            });
            if !version.is_empty() {
                body["version"] = serde_json::json!(version);
            }

            let req = self.client.post(&url)
                .bearer_auth(&self.token)
                .json(&body);

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
        let category = self.derive_category(caption, local_style);

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
            match category.as_str() {
                "Portrait" | "Wildlife" | "Nature" | "Food" | "Macro" => {
                    crate::processor::ImageStyle::Photography
                },
                "Architecture" | "Product" | "Vehicle" => {
                    if local_style == crate::processor::ImageStyle::Illustration {
                         crate::processor::ImageStyle::Illustration
                    } else {
                         crate::processor::ImageStyle::Photography
                    }
                },
                _ => {
                    local_style
                }
            }
        };

        (style, category)
    }

    pub fn derive_category(&self, caption: Option<&str>, _style: crate::processor::ImageStyle) -> String {
        let low_caps = caption.unwrap_or_default().to_lowercase();
        
        if low_caps.contains("face") || low_caps.contains("person") || low_caps.contains("man") || 
           low_caps.contains("woman") || low_caps.contains("human") || low_caps.contains("eye") || 
           low_caps.contains("skin") || low_caps.contains("portrait") || low_caps.contains("girl") || 
           low_caps.contains("boy") || low_caps.contains("selfie") {
            return "Portrait".to_string();
        }

        if low_caps.contains("animal") || low_caps.contains("deer") || low_caps.contains("fur") || 
           low_caps.contains("bird") || low_caps.contains("pet") || low_caps.contains("dog") || 
           low_caps.contains("cat") || low_caps.contains("wildlife") || low_caps.contains("horse") || 
           low_caps.contains("lion") || low_caps.contains("tiger") || low_caps.contains("fish") || 
           low_caps.contains("insect") || low_caps.contains("feathers") || low_caps.contains("mammal") {
            return "Wildlife".to_string();
        }

        if low_caps.contains("tree") || low_caps.contains("forest") || low_caps.contains("mountain") || 
           low_caps.contains("sky") || low_caps.contains("grass") || low_caps.contains("field") || 
           low_caps.contains("landscape") || low_caps.contains("outdoors") || 
           low_caps.contains("flower") || low_caps.contains("leaf") || low_caps.contains("plant") ||
           low_caps.contains("beach") || low_caps.contains("ocean") || low_caps.contains("river") ||
           low_caps.contains("cloud") || low_caps.contains("sunset") || low_caps.contains("nature") {
            return "Nature".to_string();
        }

        if low_caps.contains("building") || low_caps.contains("house") || low_caps.contains("street") || 
           low_caps.contains("city") || low_caps.contains("room") || low_caps.contains("interior") ||
           low_caps.contains("architecture") || low_caps.contains("window") || low_caps.contains("door") ||
           low_caps.contains("tower") || low_caps.contains("bridge") || low_caps.contains("temple") {
            return "Architecture".to_string();
        }

        "Other".to_string()
    }
}

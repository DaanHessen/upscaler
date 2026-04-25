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
    pub fn new() -> Self {
        Self {
            client: Client::builder().timeout(Duration::from_secs(300)).build().unwrap(),
            token: env::var("REPLICATE_API_TOKEN").unwrap_or_default(),
        }
    }

    pub async fn run_topaz(&self, image_url: &str, upscale_factor: &str, style: &str, topaz_mode: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let enhance_model = match topaz_mode {
            "Low Quality Recovery" => "Low Resolution V2",
            "Standard" => "Standard V2",
            "High Fidelity" => "High Fidelity V2",
            "CGI" => "CGI",
            "Text Refine" => "Text Refine",
            _ => {
                // Fallback
                if style == "PHOTOGRAPHY" { "Standard V2" } else { "CGI" }
            }
        };
        let subject_detection = "None"; // More robust against artifacts than Foreground
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "enhance_model": enhance_model,
                "upscale_factor": upscale_factor,
                "face_enhancement": false,
                "subject_detection": subject_detection,
            }
        });

        info!("Starting Replicate Topaz job...");
        let resp = self.client.post("https://api.replicate.com/v1/models/topazlabs/image-upscale/predictions")
            .bearer_auth(&self.token)
            .json(&req_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let txt = resp.text().await?;
            return Err(format!("Replicate API error: {}", txt).into());
        }

        let mut pred: ReplicatePredictionResponse = resp.json().await?;
        
        // Poll for completion
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
                if let Some(first) = out_arr.first().and_then(|v| v.as_str()) {
                    return Ok(first.to_string());
                }
            }
            Err("No output from Replicate".into())
        } else {
            Err("No output from Replicate".into())
        }
    }

    pub async fn run_nafnet(&self, image_url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "task_type": "Image Debluring (REDS)"
            }
        });
        self.run_replicate_model("megvii-research/nafnet", "018241a6c880319404eaa2714b764313e27e11f950a7ff0a7b5b37b27b74dcf7", req_body).await
    }

    pub async fn run_scunet(&self, image_url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let req_body = serde_json::json!({
            "input": {
                "image": image_url,
                "model_name": "real image denoising"
            }
        });
        self.run_replicate_model("cszn/scunet", "b4eb5b1db3c94294246d628d09559c55b6ef2dd33c5eeb24f2b1d9fc665ed5b7", req_body).await
    }

    async fn run_replicate_model(&self, _model: &str, version: &str, req_body: serde_json::Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let resp = self.client.post("https://api.replicate.com/v1/predictions")
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "version": version,
                "input": req_body["input"]
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let txt = resp.text().await?;
            return Err(format!("Replicate API error: {}", txt).into());
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
                if let Some(first) = out_arr.first().and_then(|v| v.as_str()) {
                    return Ok(first.to_string());
                }
            }
            Err("No output from Replicate".into())
        } else {
            Err("No output from Replicate".into())
        }
    }
}

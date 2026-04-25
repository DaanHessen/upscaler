use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::env;
use std::time::Duration;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct ReplicatePredictionResponse {
    pub id: String,
    pub status: String,
    pub output: Option<String>,
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

    pub async fn run_topaz(&self, image_url: &str, upscale_factor: &str, style: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let enhance_model = if style == "PHOTOGRAPHY" { "Standard V2" } else { "CGI" };
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

        if let Some(out_url) = pred.output {
            Ok(out_url)
        } else {
            Err("No output from Replicate".into())
        }
    }
}

use crate::models::{
    GenerateContentRequest, GenerateContentResponse,
};
use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use tracing::{info, error};

pub struct VertexClient {
    http_client: Client,
    project_id: String,
    location: String,
}

impl VertexClient {
    pub fn new(project_id: String, location: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120)) // Gemini image generation can take up to 90s
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            project_id,
            location,
        }
    }

    pub async fn generate_image(
        &self,
        token: &str,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-3-pro-image-preview:generateContent",
            self.project_id, self.location
        );

        info!("Sending request to Gemini API...");

        let response = self.http_client
            .post(&url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("Upstream AI provider error ({}): {}", status, error_text);
            return Err("An internal processing error occurred while generating the image.".into());
        }

        info!("Gemini API responded successfully");
        let result = response.json::<GenerateContentResponse>().await?;
        Ok(result)
    }
}

use crate::models::{
    GenerateContentRequest, GenerateContentResponse,
};
use reqwest::Client;
use std::error::Error;

pub struct VertexClient {
    http_client: Client,
    project_id: String,
    location: String,
}

impl VertexClient {
    pub fn new(project_id: String, location: String) -> Self {
        Self {
            http_client: Client::new(),
            project_id,
            location,
        }
    }

    pub async fn generate_image(
        &self,
        token: &str,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Box<dyn Error + Send + Sync>> {
        let host = "aiplatform.googleapis.com";

        let url = format!(
            "https://{}/v1/projects/{}/locations/{}/publishers/google/models/gemini-3-pro-image-preview:generateContent",
            host, self.project_id, self.location
        );

        let response = self.http_client
            .post(url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        let result = response.json::<GenerateContentResponse>().await?;
        Ok(result)
    }
}

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

#[axum::async_trait]
pub trait VertexProvider: Send + Sync {
    async fn generate_image(
        &self,
        token: &str,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Box<dyn Error + Send + Sync>>;
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
}

#[axum::async_trait]
impl VertexProvider for VertexClient {
    async fn generate_image(
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

pub struct MockVertexClient;

#[axum::async_trait]
impl VertexProvider for MockVertexClient {
    async fn generate_image(
        &self,
        _token: &str,
        _request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Box<dyn Error + Send + Sync>> {
        info!("[MOCK] VertexClient: Simulating image generation...");
        
        // Return a dummy but valid response
        use crate::models::{Candidate, Content, Part, InlineData, UsageMetadata};
        
        // 1x1 pixels black PNG in base64
        let dummy_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        
        Ok(GenerateContentResponse {
            candidates: vec![Candidate {
                content: Content {
                    role: "model".to_string(),
                    parts: vec![Part {
                        text: None,
                        inline_data: Some(InlineData {
                            mime_type: "image/png".to_string(),
                            data: dummy_image.to_string(),
                        }),
                    }],
                },
                finish_reason: "STOP".to_string(),
            }],
            usage_metadata: UsageMetadata {
                prompt_token_count: 100,
                candidates_token_count: 256,
                total_token_count: 356,
            },
        })
    }
}

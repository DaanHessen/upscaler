use aws_sdk_s3::{Client, config::Region};
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use std::error::Error;
use std::env;

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
}

impl StorageService {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let access_key = env::var("S3_ACCESS_KEY")?;
        let secret_key = env::var("S3_SECRET_KEY")?;
        let endpoint = env::var("S3_ENDPOINT")?;
        let bucket = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "upscales".to_string());

        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "supabase",
        );

        let region = Region::new("us-east-1"); // Supabase dummy region

        let config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(region)
            .endpoint_url(endpoint)
            .force_path_style(true) // CRITICAL FOR SUPABASE
            .build();

        let client = Client::from_conf(config);

        Ok(Self { client, bucket })
    }

    pub async fn upload_object(&self, path: &str, body: Vec<u8>, mime: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(body.into())
            .content_type(mime)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_signed_url(&self, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let expires_in = Duration::from_secs(3600); // 1 hour
        let presigned_request = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        Ok(presigned_request.uri().to_string())
    }
}

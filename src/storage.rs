use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use std::error::Error;
use std::env;
use tracing::{info, error};

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
    endpoint: String,
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

        let region = Region::new("us-east-1"); // Supabase requires a region but ignores the value

        let config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(region)
            .endpoint_url(&endpoint)
            .force_path_style(true) // Required for Supabase S3 compatibility
            .build();

        let client = Client::from_conf(config);

        info!("Storage service initialized (bucket={}, endpoint={})", bucket, endpoint);

        Ok(Self { client, bucket, endpoint })
    }

    pub async fn upload_object(&self, path: &str, body: Vec<u8>, mime: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let size = body.len();
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(body.into())
            .content_type(mime)
            .send()
            .await
            .map_err(|e| {
                // Extract detailed error info from the SDK error
                error!(
                    "S3 upload failed: bucket={}, key={}, size={} bytes, endpoint={}, error={:?}",
                    self.bucket, path, size, self.endpoint, e
                );
                Box::new(e) as Box<dyn Error + Send + Sync>
            })?;

        info!("Uploaded {} ({} bytes) to bucket '{}'", path, size, self.bucket);
        Ok(())
    }

    pub async fn get_signed_url(&self, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let expires_in = Duration::from_secs(3600); // 1 hour
        let presigned_request = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await
            .map_err(|e| {
                error!("Failed to generate signed URL: bucket={}, key={}, error={:?}", self.bucket, path, e);
                Box::new(e) as Box<dyn Error + Send + Sync>
            })?;

        Ok(presigned_request.uri().to_string())
    }

    pub async fn download_object(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| {
                error!("S3 download failed: bucket={}, key={}, error={:?}", self.bucket, path, e);
                Box::new(e) as Box<dyn Error + Send + Sync>
            })?;

        let data = resp.body.collect().await?.into_bytes();
        Ok(data.to_vec())
    }
}

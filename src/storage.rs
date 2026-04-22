use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use std::error::Error;
use std::env;
use tracing::{info, error};

#[axum::async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload_object(&self, path: &str, body: Vec<u8>, mime: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_signed_url(&self, path: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
    async fn download_object(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>;
    async fn delete_object(&self, path: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

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
}

#[axum::async_trait]
impl StorageProvider for StorageService {
    async fn upload_object(&self, path: &str, body: Vec<u8>, mime: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
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
                error!(
                    "S3 upload failed: bucket={}, key={}, size={} bytes, endpoint={}, error={:?}",
                    self.bucket, path, size, self.endpoint, e
                );
                Box::new(e) as Box<dyn Error + Send + Sync>
            })?;

        info!("Uploaded {} ({} bytes) to bucket '{}'", path, size, self.bucket);
        Ok(())
    }

    async fn get_signed_url(&self, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
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

    async fn download_object(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        info!("S3: Initiating download for key: '{}' in bucket: '{}'", path, self.bucket);
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
        info!("S3: Successfully downloaded {} bytes for key: '{}'", data.len(), path);
        Ok(data.to_vec())
    }

    async fn delete_object(&self, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| {
                error!("S3 delete failed: bucket={}, key={}, error={:?}", self.bucket, path, e);
                Box::new(e) as Box<dyn Error + Send + Sync>
            })?;

        info!("Deleted {} from bucket '{}'", path, self.bucket);
        Ok(())
    }
}

pub struct MockStorage {
    files: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self { files: std::sync::Mutex::new(std::collections::HashMap::new()) }
    }
}

#[axum::async_trait]
impl StorageProvider for MockStorage {
    async fn upload_object(&self, path: &str, body: Vec<u8>, _mime: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.files.lock().unwrap().insert(path.to_string(), body);
        Ok(())
    }

    async fn get_signed_url(&self, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(format!("https://mock-storage.local/{}", path))
    }

    async fn download_object(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let files = self.files.lock().unwrap();
        if let Some(data) = files.get(path) {
            Ok(data.clone())
        } else {
            // Return a valid 1x1 transparent PNG
            Ok(vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
                0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
                0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
                0x42, 0x60, 0x82
            ])
        }
    }

    async fn delete_object(&self, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.files.lock().unwrap().remove(path);
        Ok(())
    }
}

use std::error::Error;
use upscaler::replicate::ReplicateClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let replicate = ReplicateClient::new()?;
    let img_url = "https://replicate.delivery/pbxt/JR3pL6mY8W8u7V4P6Y7y1rY0/low_1_256x256.jpg"; // I need a public URL, or I can use the local file if I had a server.
    // Wait, I can't easily test BLIP without a public URL or uploading to S3.
    Ok(())
}

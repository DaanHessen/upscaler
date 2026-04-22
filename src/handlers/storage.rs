use std::sync::Arc;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use tracing::{error, warn};
use crate::AppState;
use crate::processor::generate_thumbnail;

/// Storage Proxy Handler
/// 
/// Retrieves assets from S3 storage. Performance is critical here.
/// Includes "self-healing" logic to generate missing thumbnails on-the-fly.
pub async fn get_storage_object(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<Response, crate::errors::ApiError> {
    
    // Sanitizing path: remove leading slashes and prevent directory traversal
    let clean_path = path.trim_start_matches('/').replace("//", "/");
    
    // 1. Primary path: Attempt to serve directly from S3
    match state.storage.download_object(&clean_path).await {
        Ok(bytes) => {
            let mime = if clean_path.ends_with(".webp") {
                "image/webp"
            } else if clean_path.ends_with(".png") {
                "image/png"
            } else if clean_path.ends_with(".jpg") || clean_path.ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "application/octet-stream"
            };

            return Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                [(axum::http::header::CACHE_CONTROL, "public, max-age=3600")],
                bytes,
            ).into_response());
        }
        Err(e) => {
            // 2. Self-Healing Logic: If a thumbnail is missing, generate it from the original on-the-fly
            if clean_path.ends_with("_thumb.webp") {
                warn!("STORAGE PROXY: Thumbnail missing for '{}'. Attempting self-healing...", clean_path);
                
                let original_path = clean_path.replace("_thumb.webp", ".png");
                let state_clone = state.clone();
                let clean_path_clone = clean_path.clone();

                // Offload CPU-heavy image processing to a blocking thread to avoid stalling the async runtime
                let result = tokio::task::spawn_blocking(move || {
                    // This happens inside a blocking thread
                    let rt = tokio::runtime::Handle::current();
                    let original_bytes = rt.block_on(state_clone.storage.download_object(&original_path)).ok()?;
                    
                    let thumb_bytes = generate_thumbnail(&original_bytes).ok()?;
                    
                    // Fire-and-forget upload of the new thumbnail for future hits
                    let storage_clone = state_clone.storage.clone();
                    let path_clone = clean_path_clone.clone();
                    let upload_bytes = thumb_bytes.clone();
                    rt.spawn(async move {
                        let _ = storage_clone.upload_object(&path_clone, upload_bytes, "image/jpeg").await;
                    });

                    Some(thumb_bytes)
                }).await.map_err(|_| crate::errors::ApiError::Internal("Processing panic".to_string()))?;

                if let Some(thumb_bytes) = result {
                    return Ok((
                        [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
                        [(axum::http::header::CACHE_CONTROL, "public, max-age=31536000")], // Cache for 1 year
                        thumb_bytes,
                    ).into_response());
                }
            }

            error!("STORAGE PROXY: Failed to download object {}: {:?}", clean_path, e);
            Err(crate::errors::ApiError::NotFound("Object not found".to_string()))
        }
    }
}

use std::sync::Arc;
use tracing::{info, error};
use crate::AppState;

/// Janitor Service (Automatic 24-hour cleanup)
/// 
/// Periodically scans the database for jobs older than 24 hours and deletes
/// their associated files from S3 storage to save costs and maintain privacy.
pub async fn janitor_service(state: Arc<AppState>) {
    info!("Janitor cleanup service started.");
    
    loop {
        info!("Janitor: Starting cleanup cycle...");

        // 1. Clean up physical files for expired upscale jobs
        match state.db.get_expired_jobs().await {
            Ok(jobs) => {
                for (id, input_path, output_path) in jobs {
                    info!("Janitor: Expiring job {}", id);
                    
                    // Delete original
                    if !input_path.is_empty() {
                        let _ = state.storage.delete_object(&input_path).await;
                    }
                    
                    // Delete processed result if it exists
                    if let Some(out) = output_path {
                        let _ = state.storage.delete_object(&out).await;
                        // Also try to delete the thumbnail
                        let thumb = out.replace(".png", "_thumb.webp");
                        let _ = state.storage.delete_object(&thumb).await;
                    }

                    // Update DB status to EXPIRED and wipe paths
                    if let Err(e) = state.db.mark_job_expired(id).await {
                        error!("Janitor: Failed to mark job {} as expired in DB: {}", id, e);
                    }
                }
            }
            Err(e) => error!("Janitor: Failed to fetch expired jobs: {}", e),
        }

        // 2. Clean up physical files for moderation rejections
        match state.db.get_expired_moderation_logs().await {
            Ok(logs) => {
                for (id, path) in logs {
                    info!("Janitor: Deleting expired moderation record {}", id);
                    let _ = state.storage.delete_object(&path).await;
                    if let Err(e) = state.db.delete_moderation_log(id).await {
                        error!("Janitor: Failed to delete moderation log {} from DB: {}", id, e);
                    }
                }
            }
            Err(e) => error!("Janitor: Failed to fetch expired moderation logs: {}", e),
        }

        info!("Janitor: Cleanup cycle complete.");
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

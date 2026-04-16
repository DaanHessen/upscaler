use upscaler::auth::{AuthProvider, JwtAuth};
use upscaler::client::VertexClient;
use upscaler::storage::StorageService;
use upscaler::db::DbService;
use upscaler::models::{
    Content, GenerateContentRequest, GenerationConfig, ImageConfig, Part, InlineData
};
use upscaler::processor::{preprocess_image, ResizeMode, is_nsfw, init_nsfw, analyze_style, ImageStyle};
use upscaler::prompts::{ILLUSTRATION_PROMPT, PHOTOGRAPHY_PROMPT};
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use axum::{
    extract::{DefaultBodyLimit, Multipart, State, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use serde::Serialize;
use tracing::{info, error};

use upscaler::AppState;

// --- Response Types ---

#[derive(Serialize)]
struct SubmitResponse {
    success: bool,
    job_id: Uuid,
}

#[derive(Serialize)]
struct PollResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

fn err_json(status: StatusCode, msg: &str) -> impl IntoResponse {
    (status, Json(ErrorResponse { success: false, error: msg.to_string() }))
}

// --- Server ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Required for rustls 0.23+ to select a crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let project_id = env::var("PROJECT_ID").expect("PROJECT_ID must be set");
    let location = env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string());
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap_or(3000);

    info!("--- Gemini Upscaler API v2 ---");
    info!("Project: {}", project_id);
    info!("Location: {}", location);
    
    info!("Initializing local NSFW moderation model...");
    init_nsfw();

    let auth = AuthProvider::new().await?;
    let client = VertexClient::new(project_id, location);
    let storage = StorageService::new().await?;
    let db = DbService::new().await?;

    // Fetch JWKS from Supabase (Fail-Fast on startup)
    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);
    
    info!("Fetching JWKS from {}...", jwks_url);
    let jwks_response = reqwest::get(&jwks_url).await
        .expect("CRITICAL: Failed to fetch JWKS. Check SUPABASE_URL and network.");
    
    if !jwks_response.status().is_success() {
        panic!("CRITICAL: JWKS fetch failed with status {}", jwks_response.status());
    }

    let jwks: jsonwebtoken::jwk::JwkSet = jwks_response.json().await
        .expect("CRITICAL: Failed to parse JWKS JSON.");

    info!("Loaded {} public key(s) from Supabase JWKS", jwks.keys.len());

    let supabase_jwt_secret = env::var("SUPABASE_JWT_SECRET").expect("SUPABASE_JWT_SECRET must be set");

    let state = Arc::new(AppState { 
        client, 
        auth, 
        storage, 
        db,
        jwks,
        supabase_jwt_secret,
    });

    // Spawn Background Queue Worker
    let worker_state = state.clone();
    tokio::spawn(async move {
        queue_worker(worker_state).await;
    });

    // Rate Limiting: 5 requests per minute per IP, burst of 5
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(12)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upscale", post(upscale_handler))
        .route("/upscales/:job_id", get(poll_upscale_handler))
        .route("/history", get(history_handler))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024)) // 15MB
        .layer(GovernorLayer { config: governor_conf })
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(ServeDir::new("frontend"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// --- Handlers ---

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "healthy" }))
}

async fn history_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    match state.db.get_user_history(user_id).await {
        Ok(records) => (StatusCode::OK, Json(records)).into_response(),
        Err(e) => {
            error!("Failed to fetch history for user {}: {}", user_id, e);
            err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load history").into_response()
        }
    }
}

async fn poll_upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    Path(job_id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    let record = match state.db.get_job_status(job_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return err_json(StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => {
            error!("Failed to fetch job {}: {}", job_id, e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    if record.user_id != user_id {
        return err_json(StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let mut response = PollResponse {
        status: record.status.clone(),
        image_url: None,
        error: record.error_msg,
    };

    if record.status == "COMPLETED" {
        if let Some(path) = record.output_path {
            match state.storage.get_signed_url(&path).await {
                Ok(url) => response.image_url = Some(url),
                Err(e) => {
                    error!("Failed to generate signed URL for completed job {}: {}", job_id, e);
                    response.error = Some("Final image generated, but failed to create download link".to_string());
                }
            }
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

async fn upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    let mut image_data = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            image_data = field.bytes().await.ok();
            break;
        }
    }

    let data = match image_data {
        Some(d) => d.to_vec(),
        None => return err_json(StatusCode::BAD_REQUEST, "Missing 'image' field").into_response(),
    };

    // Decode image once for both NSFW and Style analysis
    let img = match image::load_from_memory(&data) {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to decode image: {}", e);
            return err_json(StatusCode::BAD_REQUEST, "Invalid image data").into_response();
        }
    };

    // 0. NSFW local guard
    match is_nsfw(&img) {
        Ok(true) => {
            info!("Upload rejected for user {}: NSFW content detected", user_id);
            return err_json(StatusCode::BAD_REQUEST, "Image violates content guidelines (NSFW).").into_response();
        }
        Ok(false) => {
            // Passed
        }
        Err(e) => {
            error!("Content moderation filter error: {}", e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Security check failed").into_response();
        }
    }

    // 0.1 Style Analysis
    let detected_style = analyze_style(&img);
    let style_str = match detected_style {
        ImageStyle::Illustration => "ILLUSTRATION",
        ImageStyle::Photography => "PHOTOGRAPHY",
    };

    info!("Received clean {} upload from user {} ({} bytes)", style_str, user_id, data.len());

    let original_id = Uuid::new_v4();
    let original_path = format!("{}/originals/{}.png", user_id, original_id);

    // 1. Upload original to S3 synchronously to avoid race condition with the Queue Worker
    if let Err(e) = state.storage.upload_object(&original_path, data, "image/png").await {
        error!("Failed to save original image: {}", e);
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload image").into_response();
    }

    // 2. Insert into queue as PENDING
    let job_id = match state.db.insert_job(user_id, &original_path, style_str).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to insert job into database: {}", e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to enqueue job").into_response();
        }
    };

    info!("Job {} enqueued for user {}", job_id, user_id);

    (StatusCode::ACCEPTED, Json(SubmitResponse { success: true, job_id })).into_response()
}

// --- Queue Worker ---

async fn queue_worker(state: Arc<AppState>) {
    info!("Queue worker loop started.");
    
    // Strict concurrency limit to enforce Vertex AI Quota limits
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));

    loop {
        // Try to claim a pending job
        match state.db.claim_pending_job().await {
            Ok(Some(job)) => {
                info!("Worker claimed job {}", job.id);
                // Acquire permit to process
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break, // semaphore closed
                };

                let state_clone = state.clone();

                tokio::spawn(async move {
                    if let Err(e) = process_upscale_job(&state_clone, &job).await {
                        error!("Job {} failed: {}", job.id, e);
                        if let Err(db_err) = state_clone.db.update_job_failed(job.id, &e.to_string()).await {
                            error!("Failed to update job status to FAILED for {}: {}", job.id, db_err);
                        }
                    } else {
                        info!("Job {} completed successfully.", job.id);
                    }
                    drop(permit);
                });
            }
            Ok(None) => {
                // No PENDING jobs. Sleep and pole again
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                error!("Queue worker DB poll failed: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_upscale_job(state: &Arc<AppState>, job: &upscaler::db::UpscaleRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 1. Download original image
    info!("Downloading original image for job {}", job.id);
    let original_data = state.storage.download_object(&job.input_path).await?;

    // 2. Preprocess image
    let processed = tokio::task::spawn_blocking(move || {
        preprocess_image(original_data, ResizeMode::Pad)
    }).await??;

    let system_prompt = match job.style.as_deref() {
        Some("ILLUSTRATION") => ILLUSTRATION_PROMPT,
        Some("PHOTOGRAPHY") => PHOTOGRAPHY_PROMPT,
        _ => "Perform super-resolution restore. Strictly maintain the content of the image without drifting.",
    };

    // 3. Get GCP token
    let token_data: String = state.auth.get_token().await?.as_str().to_string();

    // 4. Build and send request to Vertex
    let request = GenerateContentRequest {
        system_instruction: Some(Content {
            role: "system".to_string(),
            parts: vec![Part {
                text: Some(system_prompt.to_string()),
                inline_data: None,
            }],
        }),
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![
                Part {
                    text: Some("Perform super-resolution restore.".to_string()),
                    inline_data: None,
                },
                Part {
                    text: None,
                    inline_data: Some(InlineData {
                        mime_type: "image/jpeg".to_string(),
                        data: processed.base64_data,
                    }),
                },
            ],
        }],
        generation_config: GenerationConfig {
            response_modalities: vec!["IMAGE".to_string()],
            image_config: Some(ImageConfig {
                aspect_ratio: processed.ratio_name,
                image_size: "4K".to_string(),
            }),
            temperature: Some(0.0),
        },
    };

    info!("Sending request to Vertex AI for job {}", job.id);
    let gemini_response = state.client.generate_image(token_data.as_str(), request).await?;

    let candidate = gemini_response.candidates.first()
        .ok_or("Gemini returned no candidates")?;

    let inline_data = candidate.content.parts.iter().find_map(|p| p.inline_data.as_ref())
        .ok_or("No image data in Gemini response")?;

    let image_bytes = general_purpose::STANDARD.decode(&inline_data.data)?;

    // 5. Upload result back
    let processed_id = Uuid::new_v4();
    let processed_path = format!("{}/processed/{}.png", job.user_id, processed_id);

    info!("Uploading result to storage for job {}", job.id);
    state.storage.upload_object(&processed_path, image_bytes, "image/png").await?;

    // 6. Update database with success
    state.db.update_job_success(job.id, &processed_path).await?;

    Ok(())
}

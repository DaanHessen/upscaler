use upscaler::auth::{AuthProvider, JwtAuth};
use upscaler::client::VertexClient;
use upscaler::credits;
use upscaler::storage::StorageService;
use upscaler::db::DbService;
use upscaler::models::{
    Content, GenerateContentRequest, GenerationConfig, ImageConfig, Part, InlineData
};
use upscaler::processor::{preprocess_image, preprocess_image_internal, ResizeMode, is_nsfw, init_nsfw, analyze_style, ImageStyle};
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{StatusCode, HeaderMap, HeaderValue},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use serde::{Serialize, Deserialize};
use tracing::{info, error, warn};

use upscaler::AppState;

// --- Response Types ---

#[derive(Serialize)]
struct SubmitResponse {
    success: bool,
    job_id: Uuid,
    final_style: String,
}

#[derive(Serialize)]
struct ModerateResponse {
    nsfw: bool,
    detected_style: String,
    preview_base64: Option<String>,
}

#[derive(Serialize)]
struct PollResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_settings: Option<upscaler::prompts::PromptSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct HistoryItem {
    pub id: Uuid,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub quality: String,
    pub style: Option<String>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub prompt_settings: serde_json::Value,
    pub usage_metadata: serde_json::Value,
    pub latency_ms: i32,
    pub credits_charged: i32,
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

    let config = upscaler::config::Config::load()?;

    info!("--- UPSYL API v2 ---");
    info!("Project: {}", config.project_id);
    info!("Location: {}", config.location);
    
    info!("Initializing local NSFW moderation model...");
    init_nsfw();

    let auth = AuthProvider::new().await?;
    let client = Arc::new(VertexClient::new(config.project_id.clone(), config.location.clone()));
    let storage = Arc::new(StorageService::new().await?);
    let db = Arc::new(DbService::new().await?);

    // Fetch JWKS from Supabase (Fail-Fast on startup)
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", config.supabase_url);
    
    info!("Fetching JWKS from {}...", jwks_url);
    let jwks_response = reqwest::get(&jwks_url).await
        .expect("CRITICAL: Failed to fetch JWKS. Check SUPABASE_URL and network.");
    
    if !jwks_response.status().is_success() {
        panic!("CRITICAL: JWKS fetch failed with status {}", jwks_response.status());
    }

    let jwks: jsonwebtoken::jwk::JwkSet = jwks_response.json().await
        .expect("CRITICAL: Failed to parse JWKS JSON.");

    info!("Loaded {} public key(s) from Supabase JWKS", jwks.keys.len());

    let state = Arc::new(AppState { 
        client, 
        auth, 
        storage, 
        db,
        jwks,
        config: config.clone(),
    });

    // Spawn Background Services
    let worker_state = state.clone();
    tokio::spawn(async move {
        queue_worker(worker_state).await;
    });

    let janitor_state = state.clone();
    tokio::spawn(async move {
        janitor_service(janitor_state).await;
    });

    // Rate Limiting: High limits for local development to prevent blocking concurrent assets
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(50)
            .burst_size(100)
            .finish()
            .unwrap(),
    );

    // --- Routes ---
    // Authenticated + rate-limited routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/moderate", post(moderate_handler))
        .route("/upscale", post(upscale_handler))
        .route("/upscales/:job_id", get(poll_upscale_handler))
        .route("/history", get(history_handler))
        .route("/balance", get(balance_handler))
        .route("/storage/view/*path", get(get_storage_object))
        .route("/checkout", post(checkout_handler))
        .route("/auth/change-password", post(change_password_handler))
        .route("/admin/insights", get(admin_insights_handler))
        .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024)) // 25MB
        .layer(GovernorLayer { config: governor_conf });

    // Stripe webhook — NO auth, NO rate-limit (Stripe sends raw JSON with its own signature)
    let webhook_routes = Router::new()
        .route("/stripe/webhook", post(stripe_webhook_handler))
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api", api_routes)
        .merge(webhook_routes)
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("frontend"))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Termination signal received. Starting graceful shutdown...");
}

// --- Handlers ---

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "healthy" }))
}

async fn admin_insights_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<Response, upscaler::errors::ApiError> {
    if !jwt.is_admin(&state) {
        return Err(upscaler::errors::ApiError::Forbidden("Admin access required".to_string()));
    }

    match state.db.get_recent_moderation_logs().await {
        Ok(logs) => {
            let mut enriched = Vec::new();
            for mut log in logs {
                if let Some(path) = log.get("path").and_then(|p| p.as_str()) {
                    if let Ok(url) = state.storage.get_signed_url(path).await {
                        if let Some(obj) = log.as_object_mut() {
                            obj.insert("url".to_string(), serde_json::Value::String(url));
                        }
                    }
                }
                enriched.push(log);
            }
            Ok((StatusCode::OK, Json(enriched)).into_response())
        },
        Err(e) => {
            error!("Failed to fetch moderation logs: {}", e);
            Err(upscaler::errors::ApiError::Internal("Database error".to_string()))
        }
    }
}

async fn get_storage_object(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response, upscaler::errors::ApiError> {
    info!("Proxying storage object: {}", path);
    match state.storage.download_object(&path).await {
        Ok(bytes) => {
            let mime = if path.ends_with(".webp") {
                "image/webp"
            } else if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "application/octet-stream"
            };

            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                [(axum::http::header::CACHE_CONTROL, "public, max-age=3600")],
                bytes,
            ).into_response())
        }
        Err(e) => {
            error!("Proxy: Failed to download object {}: {:?}", path, e);
            Err(upscaler::errors::ApiError::NotFound("Object not found".to_string()))
        }
    }
}

async fn moderate_handler(
    State(_state): State<Arc<AppState>>,
    jwt: JwtAuth,
    mut multipart: Multipart,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
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
        None => return Err(upscaler::errors::ApiError::BadRequest("Missing 'image' field".to_string())),
    };

    // Offload CPU-heavy image processing to a blocking thread to keep the runtime responsive
    let (is_explicit, style_str, preview_b64) = match tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data).map_err(|e| e.to_string())?;
        
        // 1. Moderate first to save compute on NSFW (Audit Request)
        let is_explicit = is_nsfw(&img).unwrap_or(false);
        if is_explicit {
             return Ok::<(bool, String, Option<String>), String>((true, "SKIPPED".to_string(), None));
        }

        // 2. Only analyze style if clean
        let detected_style = analyze_style(&img, Some(&data));
        let style_str = match detected_style {
            ImageStyle::Illustration => "ILLUSTRATION",
            ImageStyle::Photography => "PHOTOGRAPHY",
        };

        // 3. NEW: Generate 1MP preview for deterministic frontend display
        let preview = preprocess_image_internal(img, ResizeMode::Pad).map_err(|e| e.to_string())?;
        
        Ok::<(bool, String, Option<String>), String>((false, style_str.to_string(), Some(preview.base64_data)))
    }).await {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            error!("Failed to process image for moderation (user {}): {}", user_id, e);
            return Err(upscaler::errors::ApiError::BadRequest("Invalid image data".to_string()));
        }
        Err(e) => {
            error!("Moderation thread panic for user {}: {}", user_id, e);
            return Err(upscaler::errors::ApiError::Internal("Processing error".to_string()));
        }
    };

    Ok((StatusCode::OK, Json(ModerateResponse {
        nsfw: is_explicit,
        detected_style: style_str,
        preview_base64: preview_b64,
    })).into_response())
}

// --- Credit & Stripe Handlers ---

async fn balance_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    // Auto-create user row if first visit
    if let Err(e) = state.db.ensure_user_exists(user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return Err(upscaler::errors::ApiError::Internal("Database error".to_string()));
    }

    match state.db.get_balance(user_id).await {
        Ok(balance) => Ok((StatusCode::OK, Json(serde_json::json!({
            "credits": balance
        }))).into_response()),
        Err(e) => {
            error!("Failed to fetch balance for user {}: {}", user_id, e);
            Err(upscaler::errors::ApiError::Internal("Failed to fetch balance".to_string()))
        }
    }
}

#[derive(Deserialize)]
struct CheckoutRequest {
    tier: String,
}

async fn checkout_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    Json(body): Json<CheckoutRequest>,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    // Ensure user exists before checkout
    if let Err(e) = state.db.ensure_user_exists(user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return Err(upscaler::errors::ApiError::Internal("Database error".to_string()));
    }

    // Build success/cancel URLs from the config
    let success_url = format!("{}/?payment=success", state.config.public_url);
    let cancel_url = format!("{}/?payment=cancelled", state.config.public_url);

    match upscaler::stripe::create_checkout_session(
        &body.tier,
        &jwt.user_id,
        &success_url,
        &cancel_url,
    ).await {
        Ok(url) => Ok((StatusCode::OK, Json(serde_json::json!({
            "url": url
        }))).into_response()),
        Err(e) => {
            error!("Stripe checkout failed for user {}: {}", user_id, e);
            Err(upscaler::errors::ApiError::Internal(format!("Checkout failed: {}", e)))
        }
    }
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    new_password: String,
}

async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    // Extract raw JWT to pass to Supabase
    let auth_header = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(h) => h,
        None => return Err(upscaler::errors::ApiError::Unauthorized("Missing authorization header".to_string())),
    };

    let client = reqwest::Client::new();
    let supabase_url = &state.config.supabase_url;
    let supabase_anon_key = &state.config.supabase_anon_key;

    if supabase_url.is_empty() || supabase_anon_key.is_empty() {
        error!("Supabase configuration missing for password change");
        return Ok(err_json(StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response());
    }

    let supabase_anon_key_val = match HeaderValue::from_str(supabase_anon_key) {
        Ok(v) => v,
        Err(_) => return Err(upscaler::errors::ApiError::Internal("Invalid config".to_string())),
    };

    let auth_header_val = match HeaderValue::from_str(auth_header) {
        Ok(v) => v,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid auth header".to_string())),
    };

    match client.put(format!("{}/auth/v1/user", supabase_url))
        .header("apikey", supabase_anon_key_val)
        .header("authorization", auth_header_val)
        .json(&serde_json::json!({
            "password": body.new_password
        }))
        .send()
        .await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Password updated successfully for user {}", user_id);
                    return Ok((StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response());
                } else {
                    let status = resp.status();
                    let err_text = resp.text().await.unwrap_or_default();
                    error!("Supabase password update failed for user {}: {} - {}", user_id, status, err_text);
                    
                    // Try to parse error from Supabase
                    let err_data: serde_json::Value = serde_json::from_str(&err_text).unwrap_or_default();
                    let msg = err_data.get("msg")
                        .or(err_data.get("error_description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Failed to update password");

                    return Err(upscaler::errors::ApiError::BadRequest(msg.to_string()));
                }
            }
            Err(e) => {
                error!("Request to Supabase failed for user {}: {}", user_id, e);
                return Err(upscaler::errors::ApiError::Internal("Failed to connect to auth provider".to_string()));
            }
        }
}

async fn stripe_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 1. Extract the Stripe-Signature header
    let sig_header = match headers.get("stripe-signature").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => {
            warn!("Stripe webhook: missing Stripe-Signature header");
            return (StatusCode::BAD_REQUEST, "Missing signature").into_response();
        }
    };

    // 2. Get webhook secret
    let webhook_secret = match env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            error!("STRIPE_WEBHOOK_SECRET not configured");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Webhook not configured").into_response();
        }
    };

    // 3. Verify signature (anti-spoofing + anti-replay)
    if let Err(e) = upscaler::stripe::verify_webhook_signature(&body, &sig_header, &webhook_secret) {
        error!("Stripe webhook signature verification failed: {}", e);
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    // 4. Parse the event
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse webhook payload: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid JSON").into_response();
        }
    };

    let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
    info!("Stripe webhook received: {}", event_type);

    // 5. Only process checkout.session.completed
    if event_type != "checkout.session.completed" {
        // Acknowledge but ignore other event types
        return (StatusCode::OK, "Event ignored").into_response();
    }

    // 6. Parse checkout data
    let checkout = match upscaler::stripe::parse_checkout_completed(&payload) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse checkout event: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid checkout event").into_response();
        }
    };

    // 7. Parse user ID
    let user_id = match Uuid::parse_str(&checkout.user_id) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid user_id in checkout metadata: {}", checkout.user_id);
            return (StatusCode::BAD_REQUEST, "Invalid user ID").into_response();
        }
    };

    // 8. Add credits (with replay protection via unique index)
    match state.db.add_credits(
        user_id,
        checkout.credits,
        &checkout.session_id,
    ).await {
        Ok(()) => {
            info!("Webhook processed: {} credits added to user {} (session: {})", 
                checkout.credits, user_id, checkout.session_id);
            (StatusCode::OK, "Credits added").into_response()
        }
        Err(e) => {
            error!("Failed to add credits for session {}: {}", checkout.session_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process payment").into_response()
        }
    }
}

async fn history_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    info!("Fetching history for user {}", user_id);
    let records = match state.db.get_user_history(user_id).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch history for user {}: {}", user_id, e);
            return Err(upscaler::errors::ApiError::Internal("Failed to load history".to_string()));
        }
    };

    let mut history = Vec::new();
    for rec in records {
        let mut item = HistoryItem {
            id: rec.id,
            status: rec.status,
            created_at: rec.created_at,
            quality: rec.quality,
            style: rec.style,
            temperature: rec.temperature,
            image_url: None,
            preview_url: None,
            error: rec.error_msg,
            prompt_settings: rec.prompt_settings,
            usage_metadata: rec.usage_metadata,
            latency_ms: rec.latency_ms,
            credits_charged: rec.credits_charged,
        };

        if item.status == "COMPLETED" {
            if let Some(path) = rec.output_path {
                // Ensure paths don't have leading slashes for the proxy
                let sanitized_path = path.trim_start_matches('/');
                item.image_url = Some(format!("/api/storage/view/{}", sanitized_path));
                
                // Generate preview URL using the naming convention worker uses
                let preview_path = sanitized_path.replace(".png", "_thumb.webp");
                item.preview_url = Some(format!("/api/storage/view/{}", preview_path));
                
                info!("History item {}: image={}, preview={}", item.id, sanitized_path, preview_path);
            }
        }
        history.push(item);
    }

    info!("Returning {} history records for user {}", history.len(), user_id);
    Ok((StatusCode::OK, Json(history)).into_response())
}

async fn poll_upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    Path(job_id): Path<Uuid>,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    match state.db.get_job_status(job_id).await {
        Ok(Some(job)) => {
            if job.user_id != user_id {
                return Err(upscaler::errors::ApiError::Forbidden("Access denied".to_string()));
            }

            let mut res = serde_json::json!({
                "status": job.status,
                "error": job.error_msg,
                "style": job.style,
            });

            if job.status == "COMPLETED" {
                if let Some(path) = &job.output_path {
                    res.as_object_mut().unwrap().insert("output_url".to_string(), serde_json::Value::String(format!("/api/storage/view/{}", path)));
                }
            }

            Ok((StatusCode::OK, Json(res)).into_response())
        }
        Ok(None) => Err(upscaler::errors::ApiError::NotFound("Job not found".to_string())),
        Err(e) => {
            error!("Failed to fetch job status: {}", e);
            Err(upscaler::errors::ApiError::Internal("Database error".to_string()))
        }
    }
}

/// Valid quality tiers that map directly to Gemini's imageSize parameter
const VALID_QUALITIES: &[&str] = &["2K", "4K"];
const DEFAULT_QUALITY: &str = "2K";
const DEFAULT_TEMPERATURE: f32 = 0.0;
const MAX_TEMPERATURE: f32 = 2.0;

async fn upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    mut multipart: Multipart,
) -> Result<Response, upscaler::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(upscaler::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    let mut image_data = None;
    let mut temperature_raw: Option<String> = None;
    let mut quality_raw: Option<String> = None;
    let mut style_override_raw: Option<String> = None;
    let mut prompt_settings_raw: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("image") => {
                image_data = field.bytes().await.ok();
            }
            Some("temperature") => {
                temperature_raw = field.text().await.ok();
            }
            Some("quality") => {
                quality_raw = field.text().await.ok();
            }
            Some("style") => {
                style_override_raw = field.text().await.ok();
            }
            Some("prompt_settings") => {
                prompt_settings_raw = field.text().await.ok();
            }
            _ => {} 
        }
    }

    let data = match image_data {
        Some(d) => d.to_vec(),
        None => return Err(upscaler::errors::ApiError::BadRequest("Missing 'image' field".to_string())),
    };

    // --- High Performance Optimization: Early Balance Check ---
    // Check if user has minimum credits before doing ANY CPU work (decoding/NSFW/Style)
    if let Err(e) = state.db.ensure_user_exists(user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return Err(upscaler::errors::ApiError::Internal("Database error".to_string()));
    }
    
    match state.db.get_balance(user_id).await {
        Ok(balance) => {
            if balance < 2 { // 2K is minimum quality now
                return Err(upscaler::errors::ApiError::PaymentRequired("Insufficient credits. Minimum upscale (2K) costs 2 credits.".to_string()));
            }
        }
        Err(e) => {
            error!("Pre-flight credit check failed for user {}: {}", user_id, e);
            return Err(upscaler::errors::ApiError::Internal("Database error".to_string()));
        }
    }

    // --- Server-side parameter validation ---
    let temperature = match &temperature_raw {
        Some(t) => {
            let parsed: f32 = t.parse().unwrap_or(DEFAULT_TEMPERATURE);
            let clamped = parsed.clamp(0.0, MAX_TEMPERATURE);
            (clamped * 10.0).round() / 10.0 
        }
        None => DEFAULT_TEMPERATURE,
    };

    let quality = match &quality_raw {
        Some(q) => {
            let upper = q.trim().to_uppercase();
            if VALID_QUALITIES.contains(&upper.as_str()) {
                upper
            } else {
                return Err(upscaler::errors::ApiError::BadRequest(format!("Invalid quality '{}'. Must be one of: 2K, 4K", q)));
            }
        }
        None => DEFAULT_QUALITY.to_string(),
    };

    let credit_cost = match quality.as_str() {
        "8k" => 4,
        "4k" => 2,
        _ => if prompt_settings_raw.as_ref().map(|s| s.contains("THINKING")).unwrap_or(false) { 2 } else { 1 },
    };

    // --- Moderation & Style Detection ---
    let data_for_processing = data.clone();
    let style_str = match tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data_for_processing).map_err(|e| e.to_string())?;
        
        let is_explicit = is_nsfw(&img).unwrap_or(false);
        if is_explicit {
             return Ok::<ProcessingOutcome, String>(ProcessingOutcome::RejectedNSFW);
        }

        let detected_style = analyze_style(&img, Some(&data_for_processing));
        let style_str = match detected_style {
            ImageStyle::Illustration => "ILLUSTRATION".to_string(),
            ImageStyle::Photography => "PHOTOGRAPHY".to_string(),
        };

        Ok::<ProcessingOutcome, String>(ProcessingOutcome::Passed { style_str })
    }).await {
        Ok(Ok(ProcessingOutcome::Passed { style_str })) => style_str,
        Ok(Ok(ProcessingOutcome::RejectedNSFW)) => {
            warn!("Upscale rejected (NSFW) for user {}", user_id);
            let _ = state.db.insert_moderation_log(user_id, "Upscale Input").await;
            return Err(upscaler::errors::ApiError::BadRequest("Image violates content guidelines (NSFW).".to_string()));
        }
        Ok(Err(e)) => {
            error!("Image processing task failed for user {}: {}", user_id, e);
            return Err(upscaler::errors::ApiError::BadRequest("Invalid image data".to_string()));
        }
        Err(e) => {
            error!("Image processing thread panic: {}", e);
            return Err(upscaler::errors::ApiError::Internal("Processing error".to_string()));
        }
    };

    let prompt_settings: upscaler::prompts::PromptSettings = match prompt_settings_raw {
        Some(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
        None => upscaler::prompts::PromptSettings::default(),
    };

    // --- Atomic credit deduction ---
    let deduct_result = state.db.deduct_credits(
        user_id,
        credit_cost,
        Uuid::nil(), 
    ).await;

    match deduct_result {
        Ok(new_balance) => {
            info!("Charged {} credits for user {} (remaining: {})", credit_cost, user_id, new_balance);
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Insufficient credits") {
                return Err(upscaler::errors::ApiError::PaymentRequired(format!("Insufficient credits. This upscale costs {} credits.", credit_cost)));
            }
            error!("Credit deduction failed for user {}: {}", user_id, e);
            return Err(upscaler::errors::ApiError::Internal("Failed to process credits".to_string()));
        }
    }

    let original_id = Uuid::new_v4();
    let original_path = format!("{}/originals/{}.png", user_id, original_id);

    if let Err(e) = state.storage.upload_object(&original_path, data, "image/png").await {
        error!("Failed to save original image: {}", e);
        if let Err(refund_err) = state.db.refund_credits(user_id, credit_cost, Uuid::nil()).await {
            error!("CRITICAL: Failed to refund {} credits to user {} after upload failure: {}", credit_cost, user_id, refund_err);
        }
        return Err(upscaler::errors::ApiError::Internal("Failed to upload image".to_string()));
    }

    let prompt_settings_json = match serde_json::to_value(&prompt_settings) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to serialize prompt settings: {}", e);
            return Err(upscaler::errors::ApiError::Internal("Serialization error".to_string()));
        }
    };

    let job_id = match state.db.insert_job(user_id, &original_path, &style_str, temperature, &quality, &prompt_settings_json).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to insert job into database: {}", e);
            if let Err(refund_err) = state.db.refund_credits(user_id, credit_cost, Uuid::nil()).await {
                error!("CRITICAL: Failed to refund {} credits to user {} after DB failure: {}", credit_cost, user_id, refund_err);
            }
            return Err(upscaler::errors::ApiError::Internal("Failed to enqueue job".to_string()));
        }
    };

    if let Err(e) = state.db.update_credits_charged(job_id, credit_cost).await {
        error!("Failed to set credits_charged on job {}: {}", job_id, e);
    }

    info!("Job {} enqueued for user {} (style={}, temp={}, quality={}, cost={})", job_id, user_id, style_str, temperature, quality, credit_cost);

    Ok((StatusCode::ACCEPTED, Json(SubmitResponse { success: true, job_id, final_style: style_str })).into_response())
}

enum ProcessingOutcome {
    Passed { style_str: String },
    RejectedNSFW,
}

// --- Queue Worker ---

async fn queue_worker(state: Arc<AppState>) {
    info!("Queue worker loop started.");
    
    // Strict concurrency limit to enforce Vertex AI Quota limits
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));

    loop {
        // 1. Wait for an available slot before even checking the DB.
        // This prevents "job hoarding" where we mark jobs as PROCESSING but can't act on them.
        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break, // semaphore closed
        };

        match state.db.claim_pending_job().await {
            Ok(Some(job)) => {
                info!("Worker claimed job {}", job.id);
                let state_clone = state.clone();                tokio::spawn(async move {
                    if let Err(e) = upscaler::worker::process_upscale_job(&state_clone, &job).await {
                        error!("Job {} failed: {}", job.id, e);
                        if let Err(db_err) = state_clone.db.update_job_failed(job.id, &e.to_string(), 0).await {
                            error!("Failed to update job status to FAILED for {}: {}", job.id, db_err);
                        }
                        // Refund credits on processing failure
                        if job.credits_charged > 0 {
                            info!("Refunding {} credits to user {} for failed job {}", job.credits_charged, job.user_id, job.id);
                            if let Err(refund_err) = state_clone.db.refund_credits(
                                job.user_id,
                                job.credits_charged,
                                job.id,
                            ).await {
                                error!("CRITICAL: Failed to refund credits for job {}: {}", job.id, refund_err);
                            }
                        }
                    } else {
                        info!("Job {} completed successfully.", job.id);
                    }
                    // Permit is dropped here when the task finishes
                    drop(permit);
                });
            }
            Ok(None) => {
                // No jobs. Drop the permit so it's available for the next iteration or other workers.
                drop(permit);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                error!("Queue worker DB poll failed: {}", e);
                drop(permit);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

// --- Janitor Service (Automatic 24-hour cleanup) ---

async fn janitor_service(state: Arc<AppState>) {
    info!("Janitor cleanup service started.");
    
    // Check for expired content every hour
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));

    loop {
        interval.tick().await;
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
    }
}

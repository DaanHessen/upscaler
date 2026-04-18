use upscaler::auth::{AuthProvider, JwtAuth};
use upscaler::client::VertexClient;
use upscaler::credits;
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
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State, Path},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
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
struct HistoryItem {
    id: Uuid,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    quality: String,
    style: Option<String>,
    temperature: f32,
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

    // Spawn Background Services
    let worker_state = state.clone();
    tokio::spawn(async move {
        queue_worker(worker_state).await;
    });

    let janitor_state = state.clone();
    tokio::spawn(async move {
        janitor_service(janitor_state).await;
    });

    // Rate Limiting: 5 requests per minute per IP, burst of 5
    // .per_second(12) means replenish 1 token every 12s (60/12 = 5 per min)
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(12)
            .burst_size(5)
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
        .route("/checkout", post(checkout_handler))
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024)) // 25MB
        .layer(GovernorLayer { config: governor_conf })
        .with_state(state.clone());

    // Stripe webhook — NO auth, NO rate-limit (Stripe sends raw JSON with its own signature)
    let webhook_routes = Router::new()
        .route("/stripe/webhook", post(stripe_webhook_handler))
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .merge(webhook_routes)
        .layer(CorsLayer::permissive())
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

async fn moderate_handler(
    State(_state): State<Arc<AppState>>,
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

    // Offload CPU-heavy image processing to a blocking thread to keep the runtime responsive
    let (is_explicit, style_str) = match tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data).map_err(|e| e.to_string())?;
        let is_explicit = is_nsfw(&img).unwrap_or(false);
        let detected_style = analyze_style(&img, Some(&data));
        let style_str = match detected_style {
            ImageStyle::Illustration => "ILLUSTRATION",
            ImageStyle::Photography => "PHOTOGRAPHY",
        };
        Ok::<(bool, String), String>((is_explicit, style_str.to_string()))
    }).await {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            error!("Failed to process image for moderation (user {}): {}", user_id, e);
            return err_json(StatusCode::BAD_REQUEST, "Invalid image data").into_response();
        }
        Err(e) => {
            error!("Moderation thread panic for user {}: {}", user_id, e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Processing error").into_response();
        }
    };

    (StatusCode::OK, Json(ModerateResponse {
        nsfw: is_explicit,
        detected_style: style_str,
    })).into_response()
}

// --- Credit & Stripe Handlers ---

async fn balance_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    // Auto-create user row if first visit
    if let Err(e) = credits::ensure_user_exists(state.db.pool(), user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    match credits::get_balance(state.db.pool(), user_id).await {
        Ok(balance) => (StatusCode::OK, Json(serde_json::json!({
            "credits": balance
        }))).into_response(),
        Err(e) => {
            error!("Failed to fetch balance for user {}: {}", user_id, e);
            err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch balance").into_response()
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
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    // Ensure user exists before checkout
    if let Err(e) = credits::ensure_user_exists(state.db.pool(), user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    // Build success/cancel URLs from the request origin
    let base_url = env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let success_url = format!("{}/?payment=success", base_url);
    let cancel_url = format!("{}/?payment=cancelled", base_url);

    match upscaler::stripe::create_checkout_session(
        &body.tier,
        &jwt.user_id,
        &success_url,
        &cancel_url,
    ).await {
        Ok(url) => (StatusCode::OK, Json(serde_json::json!({
            "url": url
        }))).into_response(),
        Err(e) => {
            error!("Stripe checkout failed for user {}: {}", user_id, e);
            err_json(StatusCode::INTERNAL_SERVER_ERROR, &format!("Checkout failed: {}", e)).into_response()
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
    match credits::add_credits(
        state.db.pool(),
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
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return err_json(StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    let records = match state.db.get_user_history(user_id).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch history for user {}: {}", user_id, e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load history").into_response();
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
            error: rec.error_msg,
        };

        if item.status == "COMPLETED" {
            if let Some(path) = rec.output_path {
                if let Ok(url) = state.storage.get_signed_url(&path).await {
                    item.image_url = Some(url);
                }
            }
        }
        history.push(item);
    }

    (StatusCode::OK, Json(history)).into_response()
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

/// Valid quality tiers that map directly to Gemini's imageSize parameter
const VALID_QUALITIES: &[&str] = &["2K", "4K"];
const DEFAULT_QUALITY: &str = "2K";
const DEFAULT_TEMPERATURE: f32 = 0.0;
const MAX_TEMPERATURE: f32 = 2.0;

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
    let mut temperature_raw: Option<String> = None;
    let mut quality_raw: Option<String> = None;
    let mut style_override_raw: Option<String> = None;

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
            _ => {} 
        }
    }

    let data = match image_data {
        Some(d) => d.to_vec(),
        None => return err_json(StatusCode::BAD_REQUEST, "Missing 'image' field").into_response(),
    };

    // --- High Performance Optimization: Early Balance Check ---
    // Check if user has minimum credits before doing ANY CPU work (decoding/NSFW/Style)
    if let Err(e) = credits::ensure_user_exists(state.db.pool(), user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }
    
    match credits::get_balance(state.db.pool(), user_id).await {
        Ok(balance) => {
            if balance < 2 { // 2K is minimum quality now
                return (StatusCode::PAYMENT_REQUIRED, Json(ErrorResponse {
                    success: false,
                    error: "Insufficient credits. Minimum upscale (2K) costs 2 credits.".to_string(),
                })).into_response();
            }
        }
        Err(e) => {
            error!("Pre-flight credit check failed for user {}: {}", user_id, e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
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
                return err_json(StatusCode::BAD_REQUEST, 
                    &format!("Invalid quality '{}'. Must be one of: 2K, 4K", q)
                ).into_response();
            }
        }
        None => DEFAULT_QUALITY.to_string(),
    };

    let credit_cost = credits::calculate_cost(&quality);

    // --- High Performance Optimization: Offload CPU work ---
    let style_override_clone = style_override_raw.clone();
    let data_clone = data.clone();
    
    let processing_result = tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data_clone).map_err(|e| e.to_string())?;
        
        // 1. NSFW detection
        let is_explicit = is_nsfw(&img).unwrap_or(false);
        if is_explicit {
            return Ok::<_, String>(ProcessingOutcome::RejectedNSFW);
        }

        // 2. Style resolution
        let style_str = match style_override_clone.as_deref().unwrap_or("AUTO").to_uppercase().as_str() {
            "ILLUSTRATION" => "ILLUSTRATION".to_string(),
            "PHOTOGRAPHY" => "PHOTOGRAPHY".to_string(),
            _ => {
                let detected_style = analyze_style(&img, Some(&data_clone));
                match detected_style {
                    ImageStyle::Illustration => "ILLUSTRATION".to_string(),
                    ImageStyle::Photography => "PHOTOGRAPHY".to_string(),
                }
            }
        };

        Ok(ProcessingOutcome::Passed { style_str })
    }).await;

    let style_str = match processing_result {
        Ok(Ok(ProcessingOutcome::Passed { style_str })) => style_str,
        Ok(Ok(ProcessingOutcome::RejectedNSFW)) => {
            info!("Upload rejected for user {}: NSFW content detected", user_id);
            // Save to insights folder for owner review and track in DB for Janitor cleanup
            let rejected_id = Uuid::new_v4();
            let rejected_path = format!("moderation/rejected/{}/{}.png", user_id, rejected_id);
            let storage = state.storage.clone();
            let db = state.db.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.upload_object(&rejected_path, data, "image/png").await {
                    error!("Failed to store rejected image for insights: {}", e);
                } else {
                    if let Err(e) = db.insert_moderation_log(user_id, &rejected_path).await {
                        error!("Failed to log moderation entry for user {}: {}", user_id, e);
                    }
                }
            });
            return err_json(StatusCode::BAD_REQUEST, "Image violates content guidelines (NSFW).").into_response();
        }
        Ok(Err(e)) => {
            error!("Image processing task failed for user {}: {}", user_id, e);
            return err_json(StatusCode::BAD_REQUEST, "Invalid image data").into_response();
        }
        Err(e) => {
            error!("Image processing thread panic: {}", e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Processing error").into_response();
        }
    };

    // --- Atomic credit deduction ---
    let deduct_result = credits::deduct_credits(
        state.db.pool(),
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
                return (StatusCode::PAYMENT_REQUIRED, Json(ErrorResponse {
                    success: false,
                    error: format!("Insufficient credits. This upscale costs {} credits.", credit_cost),
                })).into_response();
            }
            error!("Credit deduction failed for user {}: {}", user_id, e);
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to process credits").into_response();
        }
    }

    let original_id = Uuid::new_v4();
    let original_path = format!("{}/originals/{}.png", user_id, original_id);

    if let Err(e) = state.storage.upload_object(&original_path, data, "image/png").await {
        error!("Failed to save original image: {}", e);
        if let Err(refund_err) = credits::refund_credits(state.db.pool(), user_id, credit_cost, Uuid::nil()).await {
            error!("CRITICAL: Failed to refund {} credits to user {} after upload failure: {}", credit_cost, user_id, refund_err);
        }
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload image").into_response();
    }

    let job_id = match state.db.insert_job(user_id, &original_path, &style_str, temperature, &quality).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to insert job into database: {}", e);
            if let Err(refund_err) = credits::refund_credits(state.db.pool(), user_id, credit_cost, Uuid::nil()).await {
                error!("CRITICAL: Failed to refund {} credits to user {} after DB failure: {}", credit_cost, user_id, refund_err);
            }
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to enqueue job").into_response();
        }
    };

    if let Err(e) = sqlx::query("UPDATE upscales SET credits_charged = $1 WHERE id = $2")
        .bind(credit_cost)
        .bind(job_id)
        .execute(state.db.pool())
        .await {
        error!("Failed to set credits_charged on job {}: {}", job_id, e);
    }

    info!("Job {} enqueued for user {} (style={}, temp={}, quality={}, cost={})", job_id, user_id, style_str, temperature, quality, credit_cost);

    (StatusCode::ACCEPTED, Json(SubmitResponse { success: true, job_id, final_style: style_str })).into_response()
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
                let state_clone = state.clone();

                tokio::spawn(async move {
                    if let Err(e) = process_upscale_job(&state_clone, &job).await {
                        error!("Job {} failed: {}", job.id, e);
                        if let Err(db_err) = state_clone.db.update_job_failed(job.id, &e.to_string()).await {
                            error!("Failed to update job status to FAILED for {}: {}", job.id, db_err);
                        }
                        // Refund credits on processing failure
                        if job.credits_charged > 0 {
                            info!("Refunding {} credits to user {} for failed job {}", job.credits_charged, job.user_id, job.id);
                            if let Err(refund_err) = credits::refund_credits(
                                state_clone.db.pool(),
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
                image_size: job.quality.clone(),
            }),
            temperature: Some(job.temperature),
        },
    };

    info!("Sending request to Vertex AI for job {} (temp={}, quality={})", job.id, job.temperature, job.quality);
    let gemini_response = state.client.generate_image(token_data.as_str(), request).await?;

    let candidate = gemini_response.candidates.first()
        .ok_or("Gemini returned no candidates")?;

    let inline_data = candidate.content.parts.iter().find_map(|p| p.inline_data.as_ref())
        .ok_or("No image data in Gemini response")?;

    let image_bytes = general_purpose::STANDARD.decode(&inline_data.data)?;

    if candidate.finish_reason == "SAFETY" {
        return Err("Image rejected by internal safety filters.".into());
    }

    // Google Vertex AI sometimes returns a 64x64 pure black image bypass instead of explicitly tagging SAFETY
    if let Ok(generated_img) = image::load_from_memory(&image_bytes) {
        if generated_img.width() == 64 && generated_img.height() == 64 {
            return Err("Image rejected by internal safety filters.".into());
        }
    }

    // 5. Upload result back
    let processed_id = Uuid::new_v4();
    let processed_path = format!("{}/processed/{}.png", job.user_id, processed_id);

    info!("Uploading result to storage for job {}", job.id);
    state.storage.upload_object(&processed_path, image_bytes, "image/png").await?;

    // 6. Update database with success
    state.db.update_job_success(job.id, &processed_path).await?;

    Ok(())
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

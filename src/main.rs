use upscaler::auth::AuthProvider;
use upscaler::client::VertexClient;
use upscaler::storage::StorageService;
use upscaler::db::DbService;
use upscaler::processor::init_nsfw;
use upscaler::janitor::janitor_service;
use upscaler::handlers::{
    health_check, moderate_handler, balance_handler, history_handler,
    checkout_handler, stripe_webhook_handler, admin_insights_handler,
    change_password_handler, poll_upscale_handler, upscale_handler,
    storage::get_storage_object,
};

use dotenvy::dotenv;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

use upscaler::AppState;

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

    info!("--- UPSYL API v2 (MODULAR) ---");
    info!("Initializing local NSFW moderation model...");
    init_nsfw();

    let auth = AuthProvider::new().await?;
    let client = Arc::new(VertexClient::new(config.project_id.clone(), config.location.clone()));
    let storage = Arc::new(StorageService::new().await?);
    let db = Arc::new(DbService::new().await?);

    // Fetch JWKS from Supabase
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", config.supabase_url);
    let jwks_response = reqwest::get(&jwks_url).await.expect("Failed to fetch JWKS");
    let jwks: jsonwebtoken::jwk::JwkSet = jwks_response.json().await.expect("Failed to parse JWKS");

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

    // Rate Limiting
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(50)
            .burst_size(100)
            .finish()
            .unwrap(),
    );

    // --- Clean Modular Router ---
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/moderate", post(moderate_handler))
        .route("/upscale", post(upscale_handler))
        .route("/upscales/:job_id", get(poll_upscale_handler))
        .route("/history", get(history_handler))
        .route("/balance", get(balance_handler))
        .route("/checkout", post(checkout_handler))
        .route("/auth/change-password", post(change_password_handler))
        .route("/admin/insights", get(admin_insights_handler))
        .route("/storage/view/*path", get(get_storage_object))
        .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(GovernorLayer { config: governor_conf })
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/stripe/webhook", post(stripe_webhook_handler))
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("frontend"))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

async fn queue_worker(state: Arc<AppState>) {
    info!("Queue worker loop started.");
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));

    loop {
        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        match state.db.claim_pending_job().await {
            Ok(Some(job)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let _ = upscaler::worker::process_upscale_job(&state_clone, &job).await;
                    drop(permit);
                });
            }
            Ok(None) => {
                drop(permit);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(_) => {
                drop(permit);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

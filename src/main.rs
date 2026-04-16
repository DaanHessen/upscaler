use upscaler::auth::{AuthProvider, JwtAuth};
use upscaler::client::VertexClient;
use upscaler::storage::StorageService;
use upscaler::db::DbService;
use upscaler::models::{
    Content, GenerateContentRequest, GenerationConfig, ImageConfig, Part, InlineData
};
use upscaler::processor::{preprocess_image, ImageStyle, ResizeMode};
use upscaler::prompts::{ILLUSTRATION_PROMPT, PHOTOGRAPHY_PROMPT};
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use serde::Serialize;
use tracing::{info, error};

use upscaler::AppState;

#[derive(Serialize)]
struct UpscaleResponse {
    success: bool,
    image_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Required for rustls 0.23+ to select a crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let project_id = env::var("PROJECT_ID").expect("PROJECT_ID must be set");
    let location = env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string());

    info!("--- Gemini Upscaler SaaS API v1 ---");
    info!("Project: {}", project_id);
    info!("Location: {}", location);

    let auth = AuthProvider::new().await?;
    let client = VertexClient::new(project_id, location);
    let storage = StorageService::new().await?;
    let db = DbService::new().await?;

    // 1. Fetch JWKS from Supabase (Fail-Fast)
    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);
    
    info!("Fetching JWKS from {}...", jwks_url);
    let jwks_response = reqwest::get(&jwks_url).await
        .expect("CRITICAL: Failed to fetch JWKS from Supabase. Ensure SUPABASE_URL is correct and internet is available.");
    
    if !jwks_response.status().is_success() {
        let status = jwks_response.status();
        error!("JWKS Fetch failed with status: {}", status);
        panic!("CRITICAL: JWKS fetch failed with status {}. Server cannot verify tokens.", status);
    }

    let jwks: jsonwebtoken::jwk::JwkSet = jwks_response.json().await
        .expect("CRITICAL: Failed to parse JWKS JSON from Supabase.");

    info!("Successfully fetched {} public keys from Supabase", jwks.keys.len());

    let supabase_jwt_secret = env::var("SUPABASE_JWT_SECRET").expect("SUPABASE_JWT_SECRET must be set");

    let state = Arc::new(AppState { 
        client, 
        auth, 
        storage, 
        db,
        jwks,
        supabase_jwt_secret,
    });

    // 1. Rate Limiting Configuration
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(12) // 5 per minute
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    // 2. Router Setup
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upscale", post(upscale_handler))
        .layer(RequestBodyLimitLayer::new(15 * 1024 * 1024))
        .layer(GovernorLayer {
            config: governor_conf,
        })
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(ServeDir::new("frontend"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Service is healthy")
}

async fn upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth, // Authentication Guard
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid User ID format").into_response(),
    };

    let mut image_data = None;

    // 1. Extract Image
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            image_data = field.bytes().await.ok();
            break;
        }
    }

    let data = match image_data {
        Some(d) => d.to_vec(),
        None => return (StatusCode::BAD_REQUEST, "Missing 'image' field").into_response(),
    };

    // 2. Fast-Path Background Archival
    let archival_data = data.clone();
    let archival_storage = state.storage.clone();
    let original_id = Uuid::new_v4();
    let original_path = format!("{}/originals/{}.png", user_id, original_id);
    
    // Non-blocking background upload
    let archival_path = original_path.clone();
    tokio::spawn(async move {
        if let Err(e) = archival_storage.upload_object(&archival_path, archival_data, "image/png").await {
            error!("Background archival failed for user {}: {}", user_id, e);
        } else {
            info!("Successfully archived original image for user {}", user_id);
        }
    });

    // 3. Preprocess
    let processed = match preprocess_image(data, ResizeMode::Pad) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid Image: {}", e)).into_response(),
    };

    let system_prompt = match processed.style {
        ImageStyle::Illustration => ILLUSTRATION_PROMPT,
        ImageStyle::Photography => PHOTOGRAPHY_PROMPT,
    };

    // 4. Gemini Request
    let token_data: String = match state.auth.get_token().await {
        Ok(t) => t.as_str().to_string(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Auth error").into_response(),
    };

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

    match state.client.generate_image(token_data.as_str(), request).await {
        Ok(response) => {
            if let Some(candidate) = response.candidates.first() {
                for part in &candidate.content.parts {
                    if let Some(inline_data) = &part.inline_data {
                        // 5. Success! Decode and Upload Result
                        let image_bytes = match general_purpose::STANDARD.decode(&inline_data.data) {
                            Ok(b) => b,
                            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Decode error").into_response(),
                        };

                        let processed_id = Uuid::new_v4();
                        let processed_path = format!("{}/processed/{}.png", user_id, processed_id);

                        if let Err(e) = state.storage.upload_object(&processed_path, image_bytes, "image/png").await {
                           error!("Failed to upload processed image: {}", e);
                           return (StatusCode::INTERNAL_SERVER_ERROR, "Cloud storage error").into_response();
                        }

                        // 6. Record in Database
                        let style_str = match processed.style {
                            ImageStyle::Illustration => "ILLUSTRATION",
                            ImageStyle::Photography => "PHOTOGRAPHY",
                        };

                        if let Err(e) = state.db.record_upscale(user_id, style_str, &original_path, &processed_path).await {
                            error!("Database recording failed: {}", e);
                            // We don't fail the request here, but it's good to log
                        }

                        // 7. Generate Signed URL for Client
                        match state.storage.get_signed_url(&processed_path).await {
                            Ok(url) => return (StatusCode::OK, Json(UpscaleResponse { success: true, image_url: url })).into_response(),
                            Err(e) => {
                                error!("Failed to sign URL: {}", e);
                                return (StatusCode::INTERNAL_SERVER_ERROR, "URL signing failed").into_response();
                            }
                        }
                    }
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, "No image generated").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Gemini Error: {}", e)).into_response(),
    }
}

use std::sync::Arc;
use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json, Response},
};
use tracing::error;
use uuid::Uuid;
use serde::Deserialize;
use crate::AppState;
use crate::processor::{preprocess_image_internal, ResizeMode, is_nsfw, analyze_style, ImageStyle};

// --- Handlers ---

pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "healthy", "version": "2.0.0" }))
}

pub async fn moderate_handler(
    State(_state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
    mut multipart: Multipart,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
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
        None => return Err(crate::errors::ApiError::BadRequest("Missing 'image' field".to_string())),
    };

    let (is_explicit, style_str, preview_b64) = match tokio::task::spawn_blocking(move || {
        let mut reader = image::io::Reader::new(std::io::Cursor::new(&data)).with_guessed_format().map_err(|e| e.to_string())?;
        let mut limits = image::io::Limits::default();
        limits.max_alloc = Some(256 * 1024 * 1024);
        reader.limits(limits);
        let img = reader.decode().map_err(|e| e.to_string())?;
        
        let is_explicit = is_nsfw(&img).unwrap_or(false);
        if is_explicit {
             return Ok::<(bool, String, Option<String>), String>((true, "SKIPPED".to_string(), None));
        }

        let detected_style = analyze_style(&img, Some(&data));
        let style_str = match detected_style {
            ImageStyle::Illustration => "ILLUSTRATION",
            ImageStyle::Photography => "PHOTOGRAPHY",
        };

        let preview = preprocess_image_internal(img, ResizeMode::Pad).map_err(|e| e.to_string())?;
        use base64::{engine::general_purpose, Engine as _};
        let b64 = general_purpose::STANDARD.encode(&preview.jpeg_bytes);
        
        Ok::<(bool, String, Option<String>), String>((false, style_str.to_string(), Some(b64)))
    }).await {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            error!("Failed to process image for moderation (user {}): {}", user_id, e);
            return Err(crate::errors::ApiError::BadRequest("Invalid image data".to_string()));
        }
        Err(e) => {
            error!("Moderation thread panic for user {}: {}", user_id, e);
            return Err(crate::errors::ApiError::Internal("Processing error".to_string()));
        }
    };

    Ok((StatusCode::OK, Json(serde_json::json!({
        "nsfw": is_explicit,
        "detected_style": style_str,
        "preview_base64": preview_b64,
    }))).into_response())
}

pub async fn balance_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    if let Err(e) = state.db.ensure_user_exists(user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return Err(crate::errors::ApiError::Internal("Database error".to_string()));
    }

    match state.db.get_balance(user_id).await {
        Ok(balance) => Ok((StatusCode::OK, Json(serde_json::json!({
            "credits": balance
        }))).into_response()),
        Err(e) => {
            error!("Failed to fetch balance for user {}: {}", user_id, e);
            Err(crate::errors::ApiError::Internal("Failed to fetch balance".to_string()))
        }
    }
}

pub async fn history_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    let records = match state.db.get_user_history(user_id).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch history for user {}: {}", user_id, e);
            return Err(crate::errors::ApiError::Internal("Failed to load history".to_string()));
        }
    };

    let mut tasks = Vec::new();
    for rec in records {
        let state_clone = state.clone();
        tasks.push(tokio::spawn(async move {
            let mut item = serde_json::json!({
                "id": rec.id,
                "status": rec.status,
                "created_at": rec.created_at,
                "quality": rec.quality,
                "style": rec.style,
                "temperature": rec.temperature,
                "error": rec.error_msg,
                "prompt_settings": rec.prompt_settings,
                "usage_metadata": rec.usage_metadata,
                "latency_ms": rec.latency_ms,
                "credits_charged": rec.credits_charged,
                "image_url": serde_json::Value::Null,
                "preview_url": serde_json::Value::Null,
            });

            if rec.status == "COMPLETED" {
                if let Some(path) = rec.output_path {
                    let image_url_fut = state_clone.storage.get_signed_url(&path);
                    
                    let preview_path = path.replace(".png", "_thumb.jpg");
                    let preview_url_fut = state_clone.storage.get_signed_url(&preview_path);
                    
                    let (image_url_res, preview_url_res) = tokio::join!(image_url_fut, preview_url_fut);
                    
                    item["image_url"] = serde_json::Value::String(image_url_res.unwrap_or_default());
                    item["preview_url"] = serde_json::Value::String(preview_url_res.unwrap_or_default());
                }
            }
            item
        }));
    }

    let mut history = Vec::new();
    for task in tasks {
        if let Ok(item) = task.await {
            history.push(item);
        }
    }

    Ok((StatusCode::OK, Json(history)).into_response())
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub tier: String,
}

pub async fn checkout_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
    Json(body): Json<CheckoutRequest>,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    if let Err(e) = state.db.ensure_user_exists(user_id).await {
        error!("Failed to ensure user exists: {}", e);
        return Err(crate::errors::ApiError::Internal("Database error".to_string()));
    }

    let success_url = format!("{}/?payment=success", state.config.public_url);
    let cancel_url = format!("{}/?payment=cancelled", state.config.public_url);

    match crate::stripe::create_checkout_session(
        &body.tier,
        &jwt.user_id,
        &success_url,
        &cancel_url,
    ).await {
        Ok(url) => Ok((StatusCode::OK, Json(serde_json::json!({ "url": url }))).into_response()),
        Err(e) => {
            error!("Stripe checkout failed for user {}: {}", user_id, e);
            Err(crate::errors::ApiError::Internal(format!("Checkout failed: {}", e)))
        }
    }
}

pub async fn stripe_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let sig_header = match headers.get("stripe-signature").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing signature").into_response(),
    };

    let webhook_secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Webhook not configured").into_response(),
    };

    if let Err(e) = crate::stripe::verify_webhook_signature(&body, &sig_header, &webhook_secret) {
        error!("Stripe signature error: {}", e);
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
    let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");

    if event_type == "checkout.session.completed" {
        if let Ok(checkout) = crate::stripe::parse_checkout_completed(&payload) {
            if let Ok(user_id) = Uuid::parse_str(&checkout.user_id) {
                if let Err(e) = state.db.add_credits(user_id, checkout.credits, &checkout.session_id).await {
                    tracing::error!("Failed to add credits for session {}: {}", checkout.session_id, e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            }
        }
    }

    StatusCode::OK.into_response()
}

pub async fn admin_insights_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
) -> Result<Response, crate::errors::ApiError> {
    if !jwt.is_admin(&state) {
        return Err(crate::errors::ApiError::Forbidden("Admin only".to_string()));
    }

    match state.db.get_recent_moderation_logs().await {
        Ok(logs) => Ok((StatusCode::OK, Json(logs)).into_response()),
        Err(_) => Err(crate::errors::ApiError::Internal("Failed to fetch logs".to_string())),
    }
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
}

pub async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Response, crate::errors::ApiError> {
    let _user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    let auth_header = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(h) => h,
        None => return Err(crate::errors::ApiError::Unauthorized("Missing authorization header".to_string())),
    };

    let client = reqwest::Client::new();
    let supabase_url = &state.config.supabase_url;
    let supabase_anon_key = &state.config.supabase_anon_key;

    match client.put(format!("{}/auth/v1/user", supabase_url))
        .header("apikey", supabase_anon_key)
        .header("authorization", auth_header)
        .json(&serde_json::json!({ "password": body.new_password }))
        .send()
        .await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response())
                } else {
                    Err(crate::errors::ApiError::BadRequest("Failed to update password".to_string()))
                }
            }
            Err(_) => Err(crate::errors::ApiError::Internal("Auth provider error".to_string())),
        }
}

pub async fn poll_upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
    Path(job_id): Path<Uuid>,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    match state.db.get_job_status(job_id).await {
        Ok(Some(job)) => {
            if job.user_id != user_id {
                return Err(crate::errors::ApiError::Forbidden("Access denied".to_string()));
            }

            let mut res = serde_json::json!({
                "status": job.status,
                "error": job.error_msg,
                "style": job.style,
            });

            if job.status == "COMPLETED" {
                if let Some(path) = &job.output_path {
                    res["image_url"] = serde_json::Value::String(state.storage.get_signed_url(path).await.unwrap_or_default());
                    let preview_path = path.replace(".png", "_thumb.jpg");
                    res["preview_url"] = serde_json::Value::String(state.storage.get_signed_url(&preview_path).await.unwrap_or_default());
                }
                res["before_url"] = serde_json::Value::String(state.storage.get_signed_url(&job.input_path).await.unwrap_or_default());
                res["latency_ms"] = serde_json::json!(job.latency_ms);
                res["usage_metadata"] = job.usage_metadata;
                res["prompt_settings"] = job.prompt_settings;
            }

            Ok((StatusCode::OK, Json(res)).into_response())
        }
        Ok(None) => Err(crate::errors::ApiError::NotFound("Job not found".to_string())),
        Err(_) => Err(crate::errors::ApiError::Internal("Database error".to_string())),
    }
}

pub async fn upscale_handler(
    State(state): State<Arc<AppState>>,
    jwt: crate::auth::JwtAuth,
    mut multipart: Multipart,
) -> Result<Response, crate::errors::ApiError> {
    let user_id = match Uuid::parse_str(&jwt.user_id) {
        Ok(id) => id,
        Err(_) => return Err(crate::errors::ApiError::Unauthorized("Invalid user ID".to_string())),
    };

    let mut image_data = None;
    let mut quality = "2K".to_string();
    let mut style = "PHOTOGRAPHY".to_string();
    let mut temperature: f32 = 0.0;
    let mut prompt_settings_raw = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("image") => { image_data = field.bytes().await.ok(); }
            Some("quality") => { quality = field.text().await.unwrap_or_else(|_| "2K".to_string()).to_uppercase(); }
            Some("style") => { style = field.text().await.unwrap_or_else(|_| "PHOTOGRAPHY".to_string()).to_uppercase(); }
            Some("temperature") => { temperature = field.text().await.unwrap_or_default().parse().unwrap_or(0.0); }
            Some("prompt_settings") => { prompt_settings_raw = field.text().await.ok(); }
            _ => {}
        }
    }

    // Input Validation
    if !["2K", "4K"].contains(&quality.as_str()) {
        quality = "2K".to_string();
    }
    if !["PHOTOGRAPHY", "ILLUSTRATION"].contains(&style.as_str()) {
        style = "PHOTOGRAPHY".to_string();
    }
    if !temperature.is_finite() || temperature < 0.0 || temperature > 2.0 {
        temperature = 0.0;
    }

    let data = match image_data {
        Some(d) => d.to_vec(),
        None => return Err(crate::errors::ApiError::BadRequest("Missing image".to_string())),
    };

    let credit_cost = crate::credits::calculate_cost(&quality);

    // Credit check
    let balance = state.db.get_balance(user_id).await.map_err(|_| crate::errors::ApiError::Internal("DB Error".to_string()))?;
    if balance < credit_cost {
        return Err(crate::errors::ApiError::PaymentRequired("Insufficient credits".to_string()));
    }

    // Process & Moderate & Transcode
    let data_clone = data.clone();
    let style_result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let mut reader = image::io::Reader::new(std::io::Cursor::new(&data_clone)).with_guessed_format()
            .map_err(|_| "Invalid format".to_string())?;
        
        let mut limits = image::io::Limits::default();
        limits.max_alloc = Some(256 * 1024 * 1024); // Protect against decompression bombs (max 256MB)
        reader.limits(limits);
        let img = reader.decode().map_err(|_| "Decode failed".to_string())?;

        if is_nsfw(&img).unwrap_or(false) { 
            return Err("NSFW".to_string()); 
        }
        
        // Single-pass preprocessing: Resize and compress to JPEG immediately
        let processed = preprocess_image_internal(img, ResizeMode::Pad)
            .map_err(|_| "Preprocess failed".to_string())?;
        
        Ok(processed.jpeg_bytes)
    }).await.unwrap();

    let jpeg_bytes = match style_result {
        Ok(res) => res,
        Err(e) if e == "NSFW" => {
            // NSFW detected - do NOT store the image due to severe legal/liability risks (CSAM, etc).
            // We only insert a text-based log for the admin to track which user is triggering the filters.
            let _ = state.db.insert_moderation_log(user_id, "BLOCKED_IMAGE_NOT_SAVED").await;
            return Err(crate::errors::ApiError::BadRequest("NSFW detected".to_string()));
        }
        Err(_) => {
            return Err(crate::errors::ApiError::BadRequest("Invalid image data".to_string()));
        }
    };

    // Upload preprocessed image
    let job_id = Uuid::new_v4();
    let original_id = Uuid::new_v4();
    let original_path = format!("{}/originals/{}.jpg", user_id, original_id);
    
    if let Err(_) = state.storage.upload_object(&original_path, jpeg_bytes, "image/jpeg").await {
        return Err(crate::errors::ApiError::Internal("Upload error".to_string()));
    }

    let prompt_settings_json = match prompt_settings_raw {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => serde_json::Value::Null,
    };

    // Atomic Deduct and Insert
    if let Err(_) = state.db.create_job_with_deduction(job_id, user_id, &original_path, &style, temperature, &quality, &prompt_settings_json, credit_cost).await {
        let _ = state.storage.delete_object(&original_path).await; // Cleanup on DB failure
        return Err(crate::errors::ApiError::Internal("Enqueue or credit error".to_string()));
    }

    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({ "success": true, "job_id": job_id, "final_style": style }))).into_response())
}

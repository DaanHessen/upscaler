use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use std::error::Error;
use std::sync::Arc;
use gcp_auth::{Token, TokenProvider};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct JwtAuth {
    pub user_id: String,
}

pub struct AuthProvider {
    provider: Arc<dyn TokenProvider>,
}

impl AuthProvider {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let provider = gcp_auth::provider().await?;
        Ok(Self { provider })
    }

    pub async fn get_token(&self) -> Result<Token, Box<dyn Error + Send + Sync>> {
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = self.provider.token(scopes).await?;
        Ok((*token).clone())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract Bearer token
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header format".to_string()));
        }

        let token_str = &auth_header[7..];

        // 2. Decode the token header to determine algorithm and key ID
        let header = decode_header(token_str).map_err(|e| {
            error!("Failed to decode JWT header: {}", e);
            (StatusCode::UNAUTHORIZED, format!("Malformed token header: {}", e))
        })?;

        info!("JWT header: alg={:?}, kid={:?}", header.alg, header.kid);

        let app_state = Arc::<AppState>::from_ref(state);

        // 3. Build the DecodingKey based on the token's algorithm
        let decoding_key = match header.alg {
            Algorithm::HS256 => {
                // HS256: Use the Supabase JWT secret as raw bytes (it's a plain string, NOT base64)
                let secret = &app_state.supabase_jwt_secret;
                info!("Using HS256 with JWT secret ({} bytes)", secret.len());
                DecodingKey::from_secret(secret.as_bytes())
            }
            Algorithm::ES256 => {
                // ES256: Look up the public key from JWKS by key ID
                let kid = header.kid.as_ref().ok_or_else(|| {
                    error!("ES256 token is missing 'kid' header field");
                    (StatusCode::UNAUTHORIZED, "ES256 token missing key ID".to_string())
                })?;

                let jwk = app_state.jwks.find(kid).ok_or_else(|| {
                    error!("Key ID '{}' not found in JWKS (have {} keys)", kid, app_state.jwks.keys.len());
                    (StatusCode::UNAUTHORIZED, format!("Unknown key ID: {}", kid))
                })?;

                // Strip the algorithm field from the JWK to prevent internal mismatches
                // in the jsonwebtoken crate's key parsing logic.
                let mut jwk_clean = jwk.clone();
                jwk_clean.common.key_algorithm = None;

                info!("Using ES256 with JWKS key (kid={})", kid);
                DecodingKey::from_jwk(&jwk_clean).map_err(|e| {
                    error!("Failed to create DecodingKey from JWK: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "JWT key configuration error".to_string())
                })?
            }
            other => {
                warn!("Received token with unsupported algorithm: {:?}", other);
                return Err((StatusCode::UNAUTHORIZED, format!("Unsupported algorithm: {:?}", other)));
            }
        };

        // 4. Validate — only allow the exact algorithm from the token header
        let mut validation = Validation::new(header.alg);
        validation.validate_aud = false;

        // 5. Decode and verify signature
        let token_data = decode::<Claims>(token_str, &decoding_key, &validation)
            .map_err(|e| {
                error!("JWT verification failed (alg={:?}): {}", header.alg, e);
                (StatusCode::UNAUTHORIZED, format!("Token verification failed: {}", e))
            })?;

        info!("JWT verified successfully for user: {}", token_data.claims.sub);

        Ok(JwtAuth {
            user_id: token_data.claims.sub,
        })
    }
}
use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use tracing::error;
use base64::{engine::general_purpose, Engine as _};
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
        // 1. Extract Bearer token from headers
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string()));
        }

        let token_str = auth_header[7..].to_string();

        // 2. Decode header to identify algorithm and key ID
        let header = decode_header(&token_str).map_err(|e| {
            (StatusCode::UNAUTHORIZED, format!("Header error: {}", e))
        })?;

        // 3. Extract AppState using FromRef
        let app_state = Arc::<AppState>::from_ref(state);

        // 4. Prepare DecodingKey based on the algorithm in the token header
        let decoding_key = match header.alg {
            Algorithm::HS256 => {
                let secret = &app_state.supabase_jwt_secret;
                let secret_bytes = match general_purpose::STANDARD.decode(secret.trim().as_bytes()) {
                    Ok(bytes) => bytes,
                    Err(_) => secret.trim().as_bytes().to_vec(),
                };
                DecodingKey::from_secret(&secret_bytes)
            }
            Algorithm::ES256 | Algorithm::RS256 => {
                let kid = header.kid.as_ref().ok_or((StatusCode::UNAUTHORIZED, "Missing kid in token header".to_string()))?;
                let jwk = app_state.jwks.find(kid).ok_or_else(|| {
                    (StatusCode::UNAUTHORIZED, format!("Key ID {} not found in JWKS", kid))
                })?;
                
                // SURGICAL FIX: Strip the algorithm field from the JWK before parsing.
                // This prevents jsonwebtoken from failing due to internal algorithm mismatches
                // even when the key and token types are compatible.
                let mut jwk_stripped = jwk.clone();
                jwk_stripped.common.key_algorithm = None;

                DecodingKey::from_jwk(&jwk_stripped).map_err(|e| {
                    error!("DecodingKey creation failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "JWT configuration error".to_string())
                })?
            }
            _ => return Err((StatusCode::UNAUTHORIZED, format!("Unsupported algorithm: {:?}", header.alg))),
        };

        // 5. Configure validation settings
        let mut validation = Validation::new(header.alg);
        validation.validate_aud = false; 
        
        // Explicitly allow both symmetric and asymmetric algorithms
        validation.algorithms = vec![Algorithm::HS256, Algorithm::ES256, Algorithm::RS256];

        // 6. Final decode and signature verification
        let token_data = decode::<Claims>(
            &token_str,
            &decoding_key,
            &validation,
        )
        .map_err(|e| {
            error!("JWT Validation Failed: {}", e);
            (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e))
        })?;

        Ok(JwtAuth {
            user_id: token_data.claims.sub,
        })
    }
}
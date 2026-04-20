pub mod auth;
pub mod client;
pub mod credits;
pub mod models;
pub mod processor;
pub mod storage;
pub mod stripe;
pub mod db;
pub mod prompts;
pub mod worker;

use crate::client::VertexProvider;
use crate::auth::AuthProvider;
use crate::storage::StorageService;
use crate::db::DbService;
use jsonwebtoken::jwk::JwkSet;
use std::sync::Arc;

pub struct AppState {
    pub client: Arc<dyn VertexProvider>,
    pub auth: AuthProvider,
    pub storage: StorageService,
    pub db: DbService,
    pub jwks: JwkSet,
    pub supabase_jwt_secret: String,
    pub admin_user_id: Option<String>,
}

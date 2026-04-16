pub mod auth;
pub mod client;
pub mod models;
pub mod processor;
pub mod prompts;
pub mod storage;
pub mod db;

// Removed unused Arc import

use crate::client::VertexClient;
use crate::auth::AuthProvider;
use crate::storage::StorageService;
use crate::db::DbService;
use jsonwebtoken::jwk::JwkSet;

pub struct AppState {
    pub client: VertexClient,
    pub auth: AuthProvider,
    pub storage: StorageService,
    pub db: DbService,
    pub jwks: JwkSet,
    pub supabase_jwt_secret: String,
}

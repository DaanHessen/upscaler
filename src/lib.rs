pub mod config;
pub mod errors;
pub mod auth;
pub mod credits;
pub mod replicate;
pub mod models;
pub mod processor;
pub mod storage;
pub mod stripe;
pub mod db;
pub mod prompts;
pub mod worker;
pub mod janitor;
pub mod handlers;

use crate::storage::StorageProvider;
use crate::db::DbProvider;
use crate::replicate::ReplicateClient;
use jsonwebtoken::jwk::JwkSet;
use std::sync::Arc;

pub struct AppState {
    pub replicate: Arc<ReplicateClient>,
    pub storage: Arc<dyn StorageProvider>,
    pub db: Arc<dyn DbProvider>,
    pub jwks: JwkSet,
    pub config: crate::config::Config,
}

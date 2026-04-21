use std::env;
use std::error::Error;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub project_id: String,
    pub location: String,
    pub port: u16,
    pub database_url: String,
    pub supabase_url: String,
    pub supabase_jwt_secret: String,
    pub supabase_anon_key: String,
    pub stripe_webhook_secret: String,
    pub public_url: String,
    pub admin_user_id: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            project_id: env::var("PROJECT_ID").map_err(|_| "PROJECT_ID must be set")?,
            location: env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            supabase_url: env::var("SUPABASE_URL").map_err(|_| "SUPABASE_URL must be set")?,
            supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET").map_err(|_| "SUPABASE_JWT_SECRET must be set")?,
            supabase_anon_key: env::var("SUPABASE_ANON_KEY").map_err(|_| "SUPABASE_ANON_KEY must be set")?,
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").map_err(|_| "STRIPE_WEBHOOK_SECRET must be set")?,
            public_url: env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            admin_user_id: env::var("ADMIN_USER_ID").ok(),
        })
    }
}

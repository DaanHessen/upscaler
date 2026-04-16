use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::env;

#[derive(Clone)]
pub struct DbService {
    pool: PgPool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UpscaleRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub style: String,
    pub input_path: String,
    pub output_path: String,
    pub created_at: DateTime<Utc>,
}

impl DbService {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")?;
        let pool = PgPool::connect(&database_url).await?;
        
        // Ensure migrations are run
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn record_upscale(
        &self,
        user_id: Uuid,
        style: &str,
        input_path: &str,
        output_path: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query(
            "INSERT INTO upscales (user_id, style, input_path, output_path) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(user_id)
        .bind(style)
        .bind(input_path)
        .bind(output_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.get("id"))
    }
}

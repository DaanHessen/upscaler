use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::error::Error;
use std::env;
use tracing::info;

#[derive(Clone)]
pub struct DbService {
    pool: PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UpscaleRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub style: Option<String>,
    pub input_path: String,
    pub output_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub error_msg: Option<String>,
    pub temperature: f32,
    pub quality: String,
    pub credits_charged: i32,
}

impl DbService {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")?;
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database connected and migrations applied");

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn insert_job(
        &self,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        let rec: (Uuid,) = sqlx::query_as(
            "INSERT INTO upscales (user_id, input_path, style, status, temperature, quality) VALUES ($1, $2, $3, 'PENDING', $4, $5) RETURNING id"
        )
        .bind(user_id)
        .bind(input_path)
        .bind(style)
        .bind(temperature)
        .bind(quality)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.0)
    }

    pub async fn claim_pending_job(&self) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "UPDATE upscales SET status = 'PROCESSING' WHERE id = (
                SELECT id FROM upscales WHERE status = 'PENDING' ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT 1
            ) RETURNING id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged"
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    pub async fn update_job_success(
        &self,
        id: Uuid,
        output_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "UPDATE upscales SET status = 'COMPLETED', output_path = $1 WHERE id = $2"
        )
        .bind(output_path)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_job_failed(
        &self,
        id: Uuid,
        error_msg: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "UPDATE upscales SET status = 'FAILED', error_msg = $1 WHERE id = $2"
        )
        .bind(error_msg)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_job_status(
        &self,
        id: Uuid,
    ) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged FROM upscales WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    pub async fn get_user_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let records = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged FROM upscales WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

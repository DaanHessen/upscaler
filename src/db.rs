use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::error::Error;
use std::env;
use tracing::info;

#[derive(Clone, Serialize, sqlx::FromRow)]
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
    pub prompt_settings: serde_json::Value,
    pub usage_metadata: serde_json::Value,
    pub latency_ms: i32,
    pub tool_type: String,
}

#[derive(Clone)]
pub struct DbService {
    pool: PgPool,
}

#[axum::async_trait]
pub trait DbProvider: Send + Sync {
    fn pool(&self) -> &PgPool; // Only used by Postgres impl for specialty tasks, but needed for some exports. Actually, let's see.

    // Job Management
    async fn insert_job(
        &self,
        id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>>;

    async fn claim_pending_job(&self) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>>;
    
    async fn update_job_success(
        &self,
        id: Uuid,
        output_path: &str,
        usage_metadata: &serde_json::Value,
        latency_ms: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn update_job_failed(
        &self,
        id: Uuid,
        error_msg: &str,
        latency_ms: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> ;

    async fn get_job_status(&self, id: Uuid) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>>;
    async fn get_user_history(&self, user_id: Uuid) -> Result<Vec<UpscaleRecord>, Box<dyn Error + Send + Sync>>;
    async fn get_queue_position(&self, created_at: DateTime<Utc>) -> Result<i64, Box<dyn Error + Send + Sync>>;

    // Moderation & Janitor
    async fn insert_moderation_log(&self, user_id: Uuid, path: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_expired_jobs(&self) -> Result<Vec<(Uuid, String, Option<String>, String, Uuid, i32)>, Box<dyn Error + Send + Sync>>;
    async fn mark_job_expired(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_expired_moderation_logs(&self) -> Result<Vec<(Uuid, String)>, Box<dyn Error + Send + Sync>>;
    async fn delete_moderation_log(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_recent_moderation_logs(&self) -> Result<Vec<serde_json::Value>, Box<dyn Error + Send + Sync>>;

    // Credits (Moved from credits.rs to be trait-compatible)
    async fn ensure_user_exists(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_balance(&self, user_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>>;
    async fn deduct_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>>;
    async fn refund_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn add_credits(&self, user_id: Uuid, amount: i32, stripe_session_id: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn update_credits_charged(&self, job_id: Uuid, credits: i32) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn get_average_latency(&self) -> Result<i32, Box<dyn Error + Send + Sync>>;

    // Atomic combined operation
    async fn create_job_with_deduction(
        &self,
        job_id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl DbService {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")?;
        
        use sqlx::postgres::PgPoolOptions;
        use std::time::Duration;

        let pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&database_url)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database connected with optimized pool (max=50) and migrations applied");

        Ok(Self { pool })
    }
}

#[axum::async_trait]
impl DbProvider for DbService {
    fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn insert_job(
        &self,
        id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        let rec: (Uuid,) = sqlx::query_as(
            "INSERT INTO upscales (id, user_id, input_path, style, status, temperature, quality, prompt_settings, credits_charged, tool_type) VALUES ($1, $2, $3, $4, 'PENDING', $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(id)
        .bind(user_id)
        .bind(input_path)
        .bind(style)
        .bind(temperature)
        .bind(quality)
        .bind(prompt_settings)
        .bind(credits_charged)
        .bind(tool_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.0)
    }

    async fn claim_pending_job(&self) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "UPDATE upscales SET status = 'PROCESSING' WHERE id = (
                SELECT id FROM upscales WHERE status = 'PENDING' ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT 1
            ) RETURNING id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type"
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_job_success(
        &self,
        id: Uuid,
        output_path: &str,
        usage_metadata: &serde_json::Value,
        latency_ms: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "UPDATE upscales SET status = 'COMPLETED', output_path = $1, usage_metadata = $2, latency_ms = $3 WHERE id = $4"
        )
        .bind(output_path)
        .bind(usage_metadata)
        .bind(latency_ms)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_job_failed(
        &self,
        id: Uuid,
        error_msg: &str,
        latency_ms: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "UPDATE upscales SET status = 'FAILED', error_msg = $1, latency_ms = $2 WHERE id = $3"
        )
        .bind(error_msg)
        .bind(latency_ms)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_job_status(&self, id: Uuid) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type FROM upscales WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn get_user_history(&self, user_id: Uuid) -> Result<Vec<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let mut records = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status::text as status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type FROM upscales WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let topups = sqlx::query_as::<sqlx::Postgres, (Uuid, i32, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, amount, created_at FROM credit_transactions WHERE user_id = $1 AND tx_type = 'STRIPE_PURCHASE'"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        for t in topups {
            records.push(UpscaleRecord {
                id: t.0,
                user_id,
                style: None,
                input_path: String::new(),
                output_path: None,
                created_at: t.2,
                status: "COMPLETED".to_string(),
                error_msg: None,
                temperature: 0.0,
                quality: "TOP-UP".to_string(),
                credits_charged: t.1,
                prompt_settings: serde_json::json!({}),
                usage_metadata: serde_json::json!({}),
                latency_ms: 0,
                tool_type: "TOP-UP".to_string(),
            });
        }

        records.sort_by_key(|r| std::cmp::Reverse(r.created_at));
        records.truncate(50);

        Ok(records)
    }

    async fn get_queue_position(&self, created_at: DateTime<Utc>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM upscales WHERE status = 'PENDING' AND created_at < $1"
        )
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(count.0)
    }

    async fn insert_moderation_log(&self, user_id: Uuid, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO moderation_logs (user_id, path) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(path)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_expired_jobs(&self) -> Result<Vec<(Uuid, String, Option<String>, String, Uuid, i32)>, Box<dyn Error + Send + Sync>> {
        let records = sqlx::query_as::<sqlx::Postgres, (Uuid, String, Option<String>, String, Uuid, i32)>(
            "SELECT id, input_path, output_path, status::text, user_id, credits_charged FROM upscales WHERE status != 'EXPIRED' AND ((status IN ('COMPLETED', 'FAILED') AND created_at < NOW() - INTERVAL '24 hours') OR (status IN ('PENDING', 'PROCESSING') AND created_at < NOW() - INTERVAL '1 hour'))"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    async fn mark_job_expired(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "UPDATE upscales SET status = 'EXPIRED', input_path = '', output_path = NULL WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_expired_moderation_logs(&self) -> Result<Vec<(Uuid, String)>, Box<dyn Error + Send + Sync>> {
        let records = sqlx::query_as::<sqlx::Postgres, (Uuid, String)>(
            "SELECT id, path FROM moderation_logs WHERE created_at < NOW() - INTERVAL '24 hours'"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    async fn delete_moderation_log(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("DELETE FROM moderation_logs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_average_latency(&self) -> Result<i32, Box<dyn Error + Send + Sync>> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT AVG(latency_ms) FROM upscales WHERE status = 'COMPLETED' AND created_at > NOW() - INTERVAL '1 hour'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0.unwrap_or(15000.0) as i32)
    }

    async fn get_recent_moderation_logs(&self) -> Result<Vec<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        let logs = sqlx::query_as::<sqlx::Postgres, (Uuid, Uuid, String, DateTime<Utc>)>(
            "SELECT id, user_id, path, created_at FROM moderation_logs ORDER BY created_at DESC LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        let result = logs.into_iter().map(|(id, u_id, path, created)| {
            serde_json::json!({
                "id": id,
                "user_id": u_id,
                "path": path,
                "created_at": created
            })
        }).collect();

        Ok(result)
    }

    // Credits Implementation (delegating to legacy credits module logic but via trait)
    async fn ensure_user_exists(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        crate::credits::ensure_user_exists(&self.pool, user_id).await
    }
    async fn get_balance(&self, user_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>> {
        crate::credits::get_balance(&self.pool, user_id).await
    }
    async fn deduct_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>> {
        crate::credits::deduct_credits(&self.pool, user_id, amount, job_id).await
    }
    async fn refund_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        crate::credits::refund_credits(&self.pool, user_id, amount, job_id).await
    }
    async fn add_credits(&self, user_id: Uuid, amount: i32, stripe_session_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        crate::credits::add_credits(&self.pool, user_id, amount, stripe_session_id).await
    }
    async fn update_credits_charged(&self, job_id: Uuid, credits: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("UPDATE upscales SET credits_charged = $1 WHERE id = $2")
            .bind(credits)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_job_with_deduction(
        &self,
        job_id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        // 1. Lock user and check balance
        let row: (i32,) = sqlx::query_as(
            "SELECT credit_balance FROM users WHERE id = $1 FOR UPDATE"
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        let current_balance = row.0;
        if current_balance < credits_charged {
            return Err("Insufficient credits".into());
        }
        let new_balance = current_balance - credits_charged;

        // 2. Deduct credits
        sqlx::query("UPDATE users SET credit_balance = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_balance)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // 3. Ledger entry
        sqlx::query(
            "INSERT INTO credit_transactions (user_id, amount, balance_after, tx_type, reference_id, description)
             VALUES ($1, $2, $3, 'UPSCALE_DEBIT', $4, $5)"
        )
        .bind(user_id)
        .bind(-credits_charged)
        .bind(new_balance)
        .bind(job_id.to_string())
        .bind(format!("Upscale job debit ({} credits)", credits_charged))
        .execute(&mut *tx)
        .await?;

        // 4. Insert job
        sqlx::query(
            "INSERT INTO upscales (id, user_id, input_path, style, status, temperature, quality, prompt_settings, credits_charged, tool_type) VALUES ($1, $2, $3, $4, 'PENDING', $5, $6, $7, $8, $9)"
        )
        .bind(job_id)
        .bind(user_id)
        .bind(input_path)
        .bind(style)
        .bind(temperature)
        .bind(quality)
        .bind(prompt_settings)
        .bind(credits_charged)
        .bind(tool_type)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

pub struct SqliteDb {
    pool: sqlx::SqlitePool,
}

impl SqliteDb {
    pub async fn new_in_memory() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        
        // Manual schema setup for mock
        sqlx::query("CREATE TABLE users (id BLOB PRIMARY KEY, credit_balance INTEGER DEFAULT 10, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME DEFAULT CURRENT_TIMESTAMP)").execute(&pool).await?;
        sqlx::query("CREATE TABLE upscales (id BLOB PRIMARY KEY, user_id BLOB, style TEXT, input_path TEXT, output_path TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, status TEXT, error_msg TEXT, temperature REAL, quality TEXT, credits_charged INTEGER DEFAULT 0, prompt_settings TEXT, usage_metadata TEXT, tool_type TEXT DEFAULT 'UPSCALE')").execute(&pool).await?;
        sqlx::query("CREATE TABLE moderation_logs (id BLOB PRIMARY KEY, user_id BLOB, path TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)").execute(&pool).await?;
        sqlx::query("CREATE TABLE credit_transactions (id BLOB PRIMARY KEY, user_id BLOB, amount INTEGER, balance_after INTEGER, tx_type TEXT, reference_id TEXT, description TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)").execute(&pool).await?;

        Ok(Self { pool })
    }
}

#[axum::async_trait]
impl DbProvider for SqliteDb {
    fn pool(&self) -> &PgPool {
        panic!("SqliteDb does not provide a PgPool. Use trait methods only.")
    }

    async fn insert_job(
        &self,
        id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        sqlx::query("INSERT INTO upscales (id, user_id, input_path, style, status, temperature, quality, prompt_settings, credits_charged, usage_metadata, tool_type) VALUES (?, ?, ?, ?, 'PENDING', ?, ?, ?, ?, '{}', ?)")
            .bind(id)
            .bind(user_id)
            .bind(input_path)
            .bind(style)
            .bind(temperature)
            .bind(quality)
            .bind(prompt_settings.to_string())
            .bind(credits_charged)
            .bind(tool_type)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    async fn claim_pending_job(&self) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        // SQLite doesn't support FOR UPDATE SKIP LOCKED exactly the same way, 
        // but for a single-threaded test mock it's fine
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type FROM upscales WHERE status = 'PENDING' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(r) = &rec {
            sqlx::query("UPDATE upscales SET status = 'PROCESSING' WHERE id = ?").bind(r.id).execute(&self.pool).await?;
        }
        
        Ok(rec)
    }

    async fn update_job_success(&self, id: Uuid, output_path: &str, usage_metadata: &serde_json::Value, latency_ms: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("UPDATE upscales SET status = 'COMPLETED', output_path = ?, usage_metadata = ?, latency_ms = ? WHERE id = ?")
            .bind(output_path)
            .bind(usage_metadata.to_string())
            .bind(latency_ms)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_job_failed(&self, id: Uuid, error_msg: &str, latency_ms: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("UPDATE upscales SET status = 'FAILED', error_msg = ?, latency_ms = ? WHERE id = ?")
            .bind(error_msg)
            .bind(latency_ms)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_job_status(&self, id: Uuid) -> Result<Option<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type FROM upscales WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn get_user_history(&self, user_id: Uuid) -> Result<Vec<UpscaleRecord>, Box<dyn Error + Send + Sync>> {
        let mut records = sqlx::query_as::<_, UpscaleRecord>(
            "SELECT id, user_id, style, input_path, output_path, created_at, status, error_msg, temperature, quality, credits_charged, prompt_settings, usage_metadata, latency_ms, tool_type FROM upscales WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        
        let topups = sqlx::query_as::<sqlx::Sqlite, (Vec<u8>, i32, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, amount, created_at FROM credit_transactions WHERE user_id = ? AND tx_type = 'STRIPE_PURCHASE'"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        for t in topups {
            records.push(UpscaleRecord {
                id: Uuid::from_slice(&t.0).unwrap_or(Uuid::new_v4()),
                user_id,
                style: None,
                input_path: String::new(),
                output_path: None,
                created_at: t.2,
                status: "COMPLETED".to_string(),
                error_msg: None,
                temperature: 0.0,
                quality: "TOP-UP".to_string(),
                credits_charged: t.1,
                prompt_settings: serde_json::json!({}),
                usage_metadata: serde_json::json!({}),
                latency_ms: 0,
                tool_type: "TOP-UP".to_string(),
            });
        }

        records.sort_by_key(|r| std::cmp::Reverse(r.created_at));
        records.truncate(50);
        
        Ok(records)
    }

    async fn get_queue_position(&self, created_at: DateTime<Utc>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM upscales WHERE status = 'PENDING' AND created_at < ?")
            .bind(created_at)
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    async fn insert_moderation_log(&self, user_id: Uuid, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("INSERT INTO moderation_logs (id, user_id, path) VALUES (?, ?, ?)")
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_expired_jobs(&self) -> Result<Vec<(Uuid, String, Option<String>, String, Uuid, i32)>, Box<dyn Error + Send + Sync>> {
        let records = sqlx::query_as::<sqlx::Sqlite, (Uuid, String, Option<String>, String, Uuid, i32)>(
            "SELECT id, input_path, output_path, status, user_id, credits_charged FROM upscales WHERE status != 'EXPIRED' AND ((status IN ('COMPLETED', 'FAILED') AND created_at < datetime('now', '-24 hours')) OR (status IN ('PENDING', 'PROCESSING') AND created_at < datetime('now', '-1 hour')))"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    async fn mark_job_expired(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("UPDATE upscales SET status = 'EXPIRED', input_path = '', output_path = NULL WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_expired_moderation_logs(&self) -> Result<Vec<(Uuid, String)>, Box<dyn Error + Send + Sync>> {
        let records = sqlx::query_as::<sqlx::Sqlite, (Uuid, String)>(
            "SELECT id, path FROM moderation_logs WHERE created_at < datetime('now', '-24 hours')"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    async fn delete_moderation_log(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("DELETE FROM moderation_logs WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_average_latency(&self) -> Result<i32, Box<dyn Error + Send + Sync>> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT AVG(latency_ms) FROM upscales WHERE status = 'COMPLETED' AND created_at > datetime('now', '-1 hour')"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0.unwrap_or(15000.0) as i32)
    }

    async fn get_recent_moderation_logs(&self) -> Result<Vec<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        let logs = sqlx::query_as::<sqlx::Sqlite, (Uuid, Uuid, String, DateTime<Utc>)>(
            "SELECT id, user_id, path, created_at FROM moderation_logs ORDER BY created_at DESC LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs.into_iter().map(|(id, u_id, path, created)| {
            serde_json::json!({ "id": id, "user_id": u_id, "path": path, "created_at": created })
        }).collect())
    }

    async fn ensure_user_exists(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("INSERT INTO users (id) VALUES (?) ON CONFLICT (id) DO NOTHING").bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_balance(&self, user_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>> {
        self.ensure_user_exists(user_id).await?;
        let row: (i32,) = sqlx::query_as("SELECT credit_balance FROM users WHERE id = ?").bind(user_id).fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    async fn deduct_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<i32, Box<dyn Error + Send + Sync>> {
        let balance = self.get_balance(user_id).await?;
        if balance < amount { return Err("Insufficient credits".into()); }
        let new_balance = balance - amount;
        sqlx::query("UPDATE users SET credit_balance = ? WHERE id = ?").bind(new_balance).bind(user_id).execute(&self.pool).await?;
        sqlx::query("INSERT INTO credit_transactions (id, user_id, amount, balance_after, tx_type, reference_id, description) VALUES (?, ?, ?, ?, 'UPSCALE_DEBIT', ?, ?)")
            .bind(Uuid::new_v4()).bind(user_id).bind(-amount).bind(new_balance).bind(job_id.to_string()).bind("Upscale debit").execute(&self.pool).await?;
        Ok(new_balance)
    }

    async fn refund_credits(&self, user_id: Uuid, amount: i32, job_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let balance = self.get_balance(user_id).await?;
        let new_balance = balance + amount;
        sqlx::query("UPDATE users SET credit_balance = ? WHERE id = ?").bind(new_balance).bind(user_id).execute(&self.pool).await?;
        sqlx::query("INSERT INTO credit_transactions (id, user_id, amount, balance_after, tx_type, reference_id, description) VALUES (?, ?, ?, ?, 'REFUND', ?, ?)")
            .bind(Uuid::new_v4()).bind(user_id).bind(amount).bind(new_balance).bind(job_id.to_string()).bind("Refund").execute(&self.pool).await?;
        Ok(())
    }

    async fn add_credits(&self, user_id: Uuid, amount: i32, stripe_session_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let balance = self.get_balance(user_id).await?;
        let new_balance = balance + amount;
        sqlx::query("UPDATE users SET credit_balance = ? WHERE id = ?").bind(new_balance).bind(user_id).execute(&self.pool).await?;
        sqlx::query("INSERT INTO credit_transactions (id, user_id, amount, balance_after, tx_type, reference_id, description) VALUES (?, ?, ?, ?, 'STRIPE_PURCHASE', ?, ?)")
            .bind(Uuid::new_v4()).bind(user_id).bind(amount).bind(new_balance).bind(stripe_session_id).bind("Stripe purchase").execute(&self.pool).await?;
        Ok(())
    }

    async fn update_credits_charged(&self, job_id: Uuid, credits: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("UPDATE upscales SET credits_charged = ? WHERE id = ?")
            .bind(credits)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_job_with_deduction(
        &self,
        job_id: Uuid,
        user_id: Uuid,
        input_path: &str,
        style: &str,
        temperature: f32,
        quality: &str,
        prompt_settings: &serde_json::Value,
        credits_charged: i32,
        tool_type: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        // 1. Check balance
        self.ensure_user_exists(user_id).await?;
        let row: (i32,) = sqlx::query_as("SELECT credit_balance FROM users WHERE id = ?")
            .bind(user_id).fetch_one(&mut *tx).await?;

        let current_balance = row.0;
        if current_balance < credits_charged { return Err("Insufficient credits".into()); }
        let new_balance = current_balance - credits_charged;

        // 2. Deduct credits
        sqlx::query("UPDATE users SET credit_balance = ? WHERE id = ?")
            .bind(new_balance).bind(user_id).execute(&mut *tx).await?;

        // 3. Ledger entry
        sqlx::query("INSERT INTO credit_transactions (id, user_id, amount, balance_after, tx_type, reference_id, description) VALUES (?, ?, ?, ?, 'UPSCALE_DEBIT', ?, ?)")
            .bind(Uuid::new_v4()).bind(user_id).bind(-credits_charged).bind(new_balance).bind(job_id.to_string()).bind("Upscale debit").execute(&mut *tx).await?;

        // 4. Insert job
        sqlx::query("INSERT INTO upscales (id, user_id, input_path, style, status, temperature, quality, prompt_settings, credits_charged, usage_metadata, tool_type) VALUES (?, ?, ?, ?, 'PENDING', ?, ?, ?, ?, '{}', ?)")
            .bind(job_id)
            .bind(user_id)
            .bind(input_path)
            .bind(style)
            .bind(temperature)
            .bind(quality)
            .bind(prompt_settings.to_string())
            .bind(credits_charged)
            .bind(tool_type)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

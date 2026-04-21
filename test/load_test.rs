use upscaler::client::MockVertexClient;
use upscaler::prompts::{PromptSettings};
use upscaler::AppState;
use upscaler::worker::process_upscale_job;
use upscaler::auth::AuthProvider;
use upscaler::storage::StorageService;
use upscaler::db::DbService;
use std::sync::Arc;
use upscaler::db::DbProvider;
use uuid::Uuid;
use dotenvy::dotenv;
use tokio::task::JoinSet;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .init();

    println!("\n🚀 === UPSYL BACKEND LOAD TEST === 🚀\n");

    // 1. Setup Mock State (All in-memory!)
    let mock_client = Arc::new(MockVertexClient);
    let auth = AuthProvider::new_mock(); 
    let storage = Arc::new(upscaler::storage::MockStorage::new());
    let db = Arc::new(upscaler::db::SqliteDb::new_in_memory().await?);
    let jwks = jsonwebtoken::jwk::JwkSet { keys: vec![] };
    
    let state = Arc::new(AppState {
        client: mock_client,
        auth,
        storage,
        db,
        jwks,
        supabase_jwt_secret: "load-test-secret".to_string(),
        admin_user_id: None,
    });

    // 2. Mock a test user in SQLite
    let user_id = Uuid::new_v4();
    state.db.ensure_user_exists(user_id).await?;
    println!("🚀 Backend ready with mocked user: {}", user_id);

    // 3. Configuration
    let concurrency = 50;
    println!("Simulating {} concurrent processing requests...", concurrency);

    let start_time = Instant::now();
    let mut set = JoinSet::new();

    // 4. Spawn concurrent tasks
    for i in 0..concurrency {
        let state_clone = state.clone();
        let user_id_clone = user_id;
        
        set.spawn(async move {
            let job_id = Uuid::new_v4();
            // We simulate a job record that exists
            let dummy_record = upscaler::db::UpscaleRecord {
                id: job_id,
                user_id: user_id_clone,
                style: Some("PHOTOGRAPHY".to_string()),
                input_path: "test/images/original.png".to_string(),
                output_path: None,
                created_at: chrono::Utc::now(),
                status: "PROCESSING".to_string(),
                error_msg: None,
                temperature: 0.5,
                quality: "2K".to_string(),
                credits_charged: 0,
                prompt_settings: serde_json::to_value(PromptSettings::default()).unwrap(),
                usage_metadata: serde_json::json!({}),
            };
            
            // Invoke processing logic
            // Note: will fail on download if file doesn't exist, but we test the handler throughput
            if let Err(e) = process_upscale_job(&state_clone, &dummy_record).await {
                // We expect failure on download if the path is dummy, but we want to see the error type
                if e.to_string().contains("NotFound") || e.to_string().contains("NoSuchKey") {
                    return Ok::<(), String>(());
                }
                return Err::<(), String>(format!("Unexpected fail on job {}: {}", i, e));
            }
            Ok::<(), String>(())
        });
    }

    let mut successes = 0;
    let mut failures = 0;

    while let Some(res) = set.join_next().await {
        match res? {
            Ok(_) => successes += 1,
            Err(e) => {
                eprintln!("{}", e);
                failures += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    println!("\n📊 === LOAD TEST RESULTS ===");
    println!("Total Requests:  {}", concurrency);
    println!("Successes:       {}", successes);
    println!("Failures:        {}", failures);
    println!("Total Duration:  {:?}", duration);
    println!("Avg per Request: {:?}", duration / concurrency as u32);
    println!("Througput:       {:.2} req/sec", concurrency as f64 / duration.as_secs_f64());
    println!("============================\n");

    Ok(())
}

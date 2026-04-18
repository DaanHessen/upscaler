use sqlx::PgPool;
use uuid::Uuid;
use std::error::Error;
use tracing::{info, warn};

/// Credit cost per quality tier (placeholder values — will be tuned later)
pub fn calculate_cost(quality: &str) -> i32 {
    match quality {
        "2K" => 2,
        "4K" => 4,
        _ => 2, // fallback to default 2K tier
    }
}

/// Ensure a user row exists in public.users (auto-creates on first API call).
/// Uses ON CONFLICT DO NOTHING so concurrent requests don't race.
pub async fn ensure_user_exists(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    sqlx::query(
        "INSERT INTO users (id) VALUES ($1) ON CONFLICT (id) DO NOTHING"
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the current credit balance for a user.
pub async fn get_balance(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    // Ensure user exists first (idempotent)
    ensure_user_exists(pool, user_id).await?;

    let row: (i32,) = sqlx::query_as(
        "SELECT credit_balance FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Atomically deduct credits with row-level locking to prevent double-spending.
///
/// Flow:
///   1. BEGIN transaction
///   2. SELECT credit_balance FROM users WHERE id = $1 FOR UPDATE (row lock)
///   3. Check balance >= cost
///   4. UPDATE users SET credit_balance = credit_balance - cost
///   5. INSERT into credit_transactions ledger
///   6. COMMIT
///
/// Returns the new balance after deduction, or an error if insufficient funds.
pub async fn deduct_credits(
    pool: &PgPool,
    user_id: Uuid,
    amount: i32,
    job_id: Uuid,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let mut tx = pool.begin().await?;

    // Step 1: Lock the user row and read current balance
    let row: (i32,) = sqlx::query_as(
        "SELECT credit_balance FROM users WHERE id = $1 FOR UPDATE"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let current_balance = row.0;

    if current_balance < amount {
        return Err(format!(
            "Insufficient credits: have {}, need {} (quality cost)",
            current_balance, amount
        ).into());
    }

    let new_balance = current_balance - amount;

    // Step 2: Deduct
    sqlx::query(
        "UPDATE users SET credit_balance = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(new_balance)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Step 3: Ledger entry
    sqlx::query(
        "INSERT INTO credit_transactions (user_id, amount, balance_after, tx_type, reference_id, description)
         VALUES ($1, $2, $3, 'UPSCALE_DEBIT', $4, $5)"
    )
    .bind(user_id)
    .bind(-amount) // negative = debit
    .bind(new_balance)
    .bind(job_id.to_string())
    .bind(format!("Upscale job debit ({} credits)", amount))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    info!("Deducted {} credits from user {} (balance: {} → {})", amount, user_id, current_balance, new_balance);
    Ok(new_balance)
}

/// Refund credits when a job fails. This is the reverse of deduct_credits.
pub async fn refund_credits(
    pool: &PgPool,
    user_id: Uuid,
    amount: i32,
    job_id: Uuid,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut tx = pool.begin().await?;

    // Lock and read
    let row: (i32,) = sqlx::query_as(
        "SELECT credit_balance FROM users WHERE id = $1 FOR UPDATE"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let new_balance = row.0 + amount;

    // Add back
    sqlx::query(
        "UPDATE users SET credit_balance = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(new_balance)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Ledger entry
    sqlx::query(
        "INSERT INTO credit_transactions (user_id, amount, balance_after, tx_type, reference_id, description)
         VALUES ($1, $2, $3, 'REFUND', $4, $5)"
    )
    .bind(user_id)
    .bind(amount) // positive = credit
    .bind(new_balance)
    .bind(job_id.to_string())
    .bind(format!("Refund for failed job ({} credits)", amount))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    info!("Refunded {} credits to user {} (new balance: {})", amount, user_id, new_balance);
    Ok(())
}

/// Add credits from a Stripe purchase. Called from the webhook handler.
/// The unique index on reference_id WHERE tx_type = 'STRIPE_PURCHASE' prevents
/// duplicate processing of the same Stripe session (replay protection).
pub async fn add_credits(
    pool: &PgPool,
    user_id: Uuid,
    amount: i32,
    stripe_session_id: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Ensure user exists (may be first interaction)
    ensure_user_exists(pool, user_id).await?;

    let mut tx = pool.begin().await?;

    // Lock user row
    let row: (i32,) = sqlx::query_as(
        "SELECT credit_balance FROM users WHERE id = $1 FOR UPDATE"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let new_balance = row.0 + amount;

    // Update balance
    sqlx::query(
        "UPDATE users SET credit_balance = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(new_balance)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Ledger entry — unique index prevents duplicate Stripe session processing
    let result = sqlx::query(
        "INSERT INTO credit_transactions (user_id, amount, balance_after, tx_type, reference_id, description)
         VALUES ($1, $2, $3, 'STRIPE_PURCHASE', $4, $5)"
    )
    .bind(user_id)
    .bind(amount)
    .bind(new_balance)
    .bind(stripe_session_id)
    .bind(format!("Stripe purchase ({} credits)", amount))
    .execute(&mut *tx)
    .await;

    match result {
        Ok(_) => {
            tx.commit().await?;
            info!("Added {} credits to user {} via Stripe session {} (new balance: {})", 
                amount, user_id, stripe_session_id, new_balance);
            Ok(())
        }
        Err(e) => {
            // Check if this is a unique constraint violation (duplicate webhook)
            let err_str = e.to_string();
            if err_str.contains("idx_credit_tx_stripe_ref") || err_str.contains("duplicate key") {
                warn!("Duplicate Stripe session {} — already processed, ignoring", stripe_session_id);
                // Rollback is automatic on drop
                Ok(())
            } else {
                Err(Box::new(e))
            }
        }
    }
}

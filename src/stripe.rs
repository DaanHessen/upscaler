use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, error, warn};

type HmacSha256 = Hmac<Sha256>;

/// Stripe price tiers: maps a tier label to (Stripe price ID env var, credit amount)
const TIERS: &[(&str, &str, i32)] = &[
    // (tier_name, env_var for Stripe Price ID, credits awarded)
    ("5", "STRIPE_PRICE_5EUR", 50),
    ("10", "STRIPE_PRICE_10EUR", 110),
];

/// Maximum allowed clock skew for webhook timestamp verification (5 minutes)
const WEBHOOK_TOLERANCE_SECS: u64 = 300;


#[derive(Deserialize)]
struct CheckoutSessionResponse {
    pub id: String,
    pub url: Option<String>,
}


/// Create a Stripe Checkout Session for a given tier.
/// Returns the Stripe-hosted checkout URL for the client to redirect to.
pub async fn create_checkout_session(
    tier: &str,
    user_id: &str,
    success_url: &str,
    cancel_url: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let stripe_secret = env::var("STRIPE_SECRET_KEY")
        .map_err(|_| "STRIPE_SECRET_KEY not configured")?;

    // Find the tier
    let (_, price_env, credits) = TIERS.iter()
        .find(|(name, _, _)| *name == tier)
        .ok_or_else(|| format!("Unknown tier: '{}'. Valid tiers: 5, 10", tier))?;

    let price_id = env::var(price_env)
        .map_err(|_| format!("{} not configured", price_env))?;

    info!("Creating Stripe checkout session for tier {} ({} credits), user {}", tier, credits, user_id);

    let client = Client::new();

    // Stripe API uses form-encoded params
    let params = [
        ("mode", "payment".to_string()),
        ("success_url", success_url.to_string()),
        ("cancel_url", cancel_url.to_string()),
        ("line_items[0][price]", price_id),
        ("line_items[0][quantity]", "1".to_string()),
        ("metadata[user_id]", user_id.to_string()),
        ("metadata[tier]", tier.to_string()),
        ("metadata[credits]", credits.to_string()),
    ];

    let response = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth(&stripe_secret, None::<&str>)
        .form(&params)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await?;
        error!("Stripe API error ({}): {}", status, error_text);
        return Err(format!("Stripe checkout creation failed: {}", error_text).into());
    }

    let session: CheckoutSessionResponse = response.json().await?;
    let url = session.url.ok_or("Stripe returned no checkout URL")?;

    info!("Stripe checkout session created: {} -> {}", session.id, url);
    Ok(url)
}

/// Verify a Stripe webhook signature.
///
/// Stripe sends a `Stripe-Signature` header with format:
///   t=<timestamp>,v1=<signature>
///
/// The signature is HMAC-SHA256 of "<timestamp>.<raw_body>" using the webhook secret.
/// We also verify the timestamp is within WEBHOOK_TOLERANCE_SECS of now to prevent replay.
pub fn verify_webhook_signature(
    payload: &[u8],
    sig_header: &str,
    secret: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Parse the Stripe-Signature header
    let mut timestamp: Option<u64> = None;
    let mut signatures: Vec<String> = Vec::new();

    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t.parse().map_err(|_| "Invalid timestamp in Stripe signature")?);
        } else if let Some(v1) = part.strip_prefix("v1=") {
            signatures.push(v1.to_string());
        }
    }

    let timestamp = timestamp.ok_or("Missing timestamp in Stripe-Signature header")?;
    
    if signatures.is_empty() {
        return Err("No v1 signatures in Stripe-Signature header".into());
    }

    // Check timestamp freshness (anti-replay)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if now.abs_diff(timestamp) > WEBHOOK_TOLERANCE_SECS {
        warn!("Stripe webhook timestamp too old: {} (now: {})", timestamp, now);
        return Err("Webhook timestamp outside tolerance window".into());
    }

    // Compute expected signature
    let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| "Invalid webhook secret")?;
    mac.update(signed_payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    // Check if any of the provided signatures match
    // Use constant-time comparison to prevent timing attacks
    let expected_bytes = expected.as_bytes();
    let valid = signatures.iter().any(|sig| {
        let sig_bytes = sig.as_bytes();
        if sig_bytes.len() != expected_bytes.len() {
            return false;
        }
        // Simple constant-time comparison
        let mut result = 0;
        for (a, b) in sig_bytes.iter().zip(expected_bytes.iter()) {
            result |= a ^ b;
        }
        result == 0
    });

    if !valid {
        error!("Stripe webhook signature mismatch");
        return Err("Invalid webhook signature".into());
    }

    Ok(())
}

/// Parsed checkout.session.completed event data
#[derive(Debug)]
pub struct CheckoutCompleted {
    pub user_id: String,
    pub tier: String,
    pub credits: i32,
    pub session_id: String,
}

/// Extract checkout completion data from a Stripe webhook event payload.
pub fn parse_checkout_completed(
    payload: &serde_json::Value,
) -> Result<CheckoutCompleted, Box<dyn Error + Send + Sync>> {
    let event_type = payload.get("type")
        .and_then(|v| v.as_str())
        .ok_or("Missing event type")?;

    if event_type != "checkout.session.completed" {
        return Err(format!("Unexpected event type: {}", event_type).into());
    }

    let session = payload.get("data")
        .and_then(|d| d.get("object"))
        .ok_or("Missing session object in event")?;

    let session_id = session.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing session ID")?;

    let metadata = session.get("metadata")
        .ok_or("Missing metadata in session")?;

    let user_id = metadata.get("user_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing user_id in session metadata")?;

    let tier = metadata.get("tier")
        .and_then(|v| v.as_str())
        .ok_or("Missing tier in session metadata")?;

    let credits: i32 = metadata.get("credits")
        .and_then(|v| v.as_str())
        .ok_or("Missing credits in session metadata")?
        .parse()
        .map_err(|_| "Invalid credits value in metadata")?;

    // Verify payment was actually successful
    let payment_status = session.get("payment_status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if payment_status != "paid" {
        return Err(format!("Payment not completed (status: {})", payment_status).into());
    }

    Ok(CheckoutCompleted {
        user_id: user_id.to_string(),
        tier: tier.to_string(),
        credits,
        session_id: session_id.to_string(),
    })
}

/// Get credits for a tier name. Returns None for unknown tiers.
pub fn credits_for_tier(tier: &str) -> Option<i32> {
    TIERS.iter()
        .find(|(name, _, _)| *name == tier)
        .map(|(_, _, credits)| *credits)
}

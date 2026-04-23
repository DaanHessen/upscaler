use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};

const SUPABASE_URL: &str = match option_env!("SUPABASE_URL") { 
    Some(v) => v, 
    None => "https://avdchsjlsuqnmdbxlrby.supabase.co" 
};
const SUPABASE_ANON_KEY: &str = match option_env!("SUPABASE_ANON_KEY") { 
    Some(v) => v, 
    None => "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImF2ZGNoc2psc3Vxbm1kYnhscmJ5Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzYxOTQyNDcsImV4cCI6MjA5MTc3MDI0N30.GuvHDSjKige2aYlgZj1AgrvqHKahsDN3VIdf_sZl26s" 
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub user: User,
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let mut initial_session = LocalStorage::get::<Session>("sb_session").ok();
    
    // Proactive JWT validation: Check if token is expired before setting signal
    if let Some(s) = &initial_session {
        if is_token_expired(&s.access_token) {
            leptos::logging::log!("Session expired (proactive check). Clearing storage.");
            LocalStorage::delete("sb_session");
            initial_session = None;
        }
    }

    let initial_user = initial_session.as_ref().map(|s| s.user.clone());
    
    let (user, set_user) = signal(initial_user);
    let (session, set_session) = signal(initial_session);
    let initial_credits = LocalStorage::get::<i32>("telemetry_credits").ok();
    let initial_history = LocalStorage::get::<Vec<crate::api::HistoryItem>>("telemetry_history").ok();
    let initial_last_fetch = LocalStorage::get::<f64>("telemetry_last_fetch").ok();

    let (credits, set_credits) = signal(initial_credits);
    let (history, set_history) = signal(initial_history);
    let (last_fetch, set_last_fetch) = signal(initial_last_fetch);
    
    let ctx = AuthContext { 
        user, 
        session, 
        set_user, 
        set_session, 
        credits, 
        set_credits,
        history,
        set_history,
        last_fetch,
        set_last_fetch
    };

    // Centralized Telemetry Sync: Watch for session changes and sync
    Effect::new(move |_| {
        if ctx.session.get().is_some() {
            ctx.sync_telemetry(false);
        }
    });
    
    provide_context(ctx);
    
    children()
}

fn is_token_expired(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 { return true; }
    
    // Extract the exp (expiration) claim from the JWT payload
    let payload_b64 = parts[1];
    if let Ok(decoded) = decode_base64_json(payload_b64) {
        if let Some(exp) = decoded.get("exp").and_then(|v| v.as_u64()) {
            let now = (js_sys::Date::now() / 1000.0) as u64;
            return now >= exp;
        }
    }
    
    false
}

pub fn decode_base64_json(b64: &str) -> Result<serde_json::Value, String> {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    
    let decoded = URL_SAFE_NO_PAD.decode(b64.as_bytes())
        .map_err(|e| format!("Base64 decode failed: {}", e))?;
    
    serde_json::from_slice(&decoded)
        .map_err(|e| format!("JSON parse failed: {}", e))
}

#[derive(Copy, Clone)]
pub struct AuthContext {
    pub user: ReadSignal<Option<User>>,
    pub session: ReadSignal<Option<Session>>,
    pub credits: ReadSignal<Option<i32>>,
    pub history: ReadSignal<Option<Vec<crate::api::HistoryItem>>>,
    pub last_fetch: ReadSignal<Option<f64>>,
    pub set_user: WriteSignal<Option<User>>,
    pub set_session: WriteSignal<Option<Session>>,
    pub set_credits: WriteSignal<Option<i32>>,
    pub set_history: WriteSignal<Option<Vec<crate::api::HistoryItem>>>,
    pub set_last_fetch: WriteSignal<Option<f64>>,
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext must be provided")
}

impl AuthContext {
    pub async fn login(&self, email: &str, password: &str) -> Result<(), String> {
        let url = format!("{}/auth/v1/token?grant_type=password", SUPABASE_URL);
        
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = Request::post(&url)
            .header("apikey", SUPABASE_ANON_KEY)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let session: Session = resp.json().await.map_err(|e| e.to_string())?;
            self.set_user.set(Some(session.user.clone()));
            self.set_session.set(Some(session.clone()));
            let _ = LocalStorage::set("sb_session", session);
            Ok(())
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["error_description"].as_str()
                .or(err_body["msg"].as_str())
                .unwrap_or("Login failed");
            Err(msg.to_string())
        }
    }

    pub async fn signup(&self, email: &str, password: &str) -> Result<(), String> {
        let url = format!("{}/auth/v1/signup", SUPABASE_URL);
        
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = Request::post(&url)
            .header("apikey", SUPABASE_ANON_KEY)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            // If email confirmation is disabled, Supabase returns a session immediately
            if let Ok(session) = resp.json::<Session>().await {
                self.set_user.set(Some(session.user.clone()));
                self.set_session.set(Some(session.clone()));
                let _ = LocalStorage::set("sb_session", session);
            }
            Ok(())
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["msg"].as_str()
                .or(err_body["error_description"].as_str())
                .unwrap_or("Signup failed");
            Err(msg.to_string())
        }
    }

    pub fn logout(&self) {
        self.set_user.set(None);
        self.set_session.set(None);
        self.set_credits.set(None);
        self.set_history.set(None);
        self.set_last_fetch.set(None);
        LocalStorage::delete("sb_session");
    }

    pub async fn recover_password(&self, email: &str, redirect_to: &str) -> Result<(), String> {
        let url = format!("{}/auth/v1/recover", SUPABASE_URL);
        
        let body = serde_json::json!({
            "email": email,
            "gotrue_meta_security": {},
        });

        let resp = Request::post(&url)
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Redirect-To", redirect_to)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            Ok(())
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["msg"].as_str()
                .or(err_body["error_description"].as_str())
                .unwrap_or("Recovery request failed");
            Err(msg.to_string())
        }
    }

    pub async fn update_password(&self, new_password: &str) -> Result<(), String> {
        let session = self.session.get().ok_or("Not authenticated")?;
        let url = format!("{}/auth/v1/user", SUPABASE_URL);
        
        let body = serde_json::json!({
            "password": new_password,
        });

        let resp = Request::put(&url)
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", &format!("Bearer {}", session.access_token))
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            Ok(())
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["msg"].as_str()
                .or(err_body["error_description"].as_str())
                .unwrap_or("Password update failed");
            Err(msg.to_string())
        }
    }

    pub fn sync_telemetry(&self, force: bool) {
        let (token, last, has_credits) = untrack(move || {
            (
                self.session.get().map(|s| s.access_token),
                self.last_fetch.get(),
                self.credits.get().is_some()
            )
        });

        if token.is_none() { return; }

        let now = js_sys::Date::now();
        
        // 10 second cache (10,000 ms)
        if !force && last.is_some() && (now - last.unwrap() < 10_000.0) && has_credits {
            return;
        }

        let ctx = *self;
        let t_str = token.unwrap();
        
        leptos::task::spawn_local(async move {
            // Fetch credits
            if let Ok(c) = crate::api::ApiClient::get_balance(Some(&t_str)).await {
                ctx.set_credits.set(Some(c));
                let _ = LocalStorage::set("telemetry_credits", c);
            } else {
                leptos::logging::error!("Telemetry: Failed to fetch balance (is token valid?)");
            }
            
            // Fetch history
            if let Ok(h) = crate::api::ApiClient::get_history(Some(&t_str)).await {
                ctx.set_history.set(Some(h.clone()));
                let _ = LocalStorage::set("telemetry_history", h);
            } else {
                leptos::logging::error!("Telemetry: Failed to fetch history");
            }
            
            let current_time = js_sys::Date::now();
            ctx.set_last_fetch.set(Some(current_time));
            let _ = LocalStorage::set("telemetry_last_fetch", current_time);
        });
    }
}

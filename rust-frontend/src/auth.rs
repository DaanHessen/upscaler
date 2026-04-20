use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};

const SUPABASE_URL: &str = "https://avdchsjlsuqnmdbxlrby.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImF2ZGNoc2psc3Vxbm1kYnhscmJ5Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzYxOTQyNDcsImV4cCI6MjA5MTc3MDI0N30.GuvHDSjKige2aYlgZj1AgrvqHKahsDN3VIdf_sZl26s";

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
    let (credits, set_credits) = signal(Option::<i32>::None);
    let (history, set_history) = signal(Option::<Vec<crate::api::HistoryItem>>::None);
    let (last_fetch, set_last_fetch) = signal(Option::<f64>::None);
    
    provide_context(AuthContext { 
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
    });
    
    children()
}

fn is_token_expired(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 { return true; }
    
    // We only need the payload (middle part)
    let payload_b64 = parts[1];
    
    // Try to decode payload using js_sys::atob if available, or just assume it's valid if we can't.
    // However, usually we can just check if we have the payload and it looks okay.
    // For a real fix, we check the 'exp' field.
    if let Ok(decoded) = decode_base64_json(payload_b64) {
        if let Some(exp) = decoded.get("exp").and_then(|v| v.as_u64()) {
            let now = (js_sys::Date::now() / 1000.0) as u64;
            return now >= exp;
        }
    }
    
    false
}

fn decode_base64_json(b64: &str) -> Result<serde_json::Value, String> {
    // Basic base64url decoding to handle JWT payload
    let mut input = b64.replace('-', "+").replace('_', "/");
    while input.len() % 4 != 0 {
        input.push('=');
    }
    
    // Use base64-js if possible, or just a simple decode
    // Since we don't have a base64 crate, we can use window.atob via wasm-bindgen
    let window = web_sys::window().ok_or("No window")?;
    let decoded_str = window.atob(&input).map_err(|_| "Base64 decode failed".to_string())?;
    
    serde_json::from_str(&decoded_str).map_err(|e| e.to_string())
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
            // Supabase usually requires email confirmation, so we don't necessarily get a session back.
            // But we can confirm success.
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

    pub fn sync_telemetry(&self, force: bool) {
        let token = self.session.get().map(|s| s.access_token);
        if token.is_none() { return; }

        let now = js_sys::Date::now();
        let last = self.last_fetch.get();
        
        // 5 minute cache (300,000 ms)
        if !force && last.is_some() && (now - last.unwrap() < 300_000.0) && self.credits.get().is_some() {
            return;
        }

        let ctx = *self;
        let t_str = token.unwrap();
        
        leptos::task::spawn_local(async move {
            // Fetch credits
            if let Ok(c) = crate::api::ApiClient::get_balance(Some(&t_str)).await {
                ctx.set_credits.set(Some(c));
            }
            
            // Fetch history
            if let Ok(h) = crate::api::ApiClient::get_history(Some(&t_str)).await {
                ctx.set_history.set(Some(h));
            }
            
            ctx.set_last_fetch.set(Some(js_sys::Date::now()));
        });
    }
}

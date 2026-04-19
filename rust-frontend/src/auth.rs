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
    let (user, set_user) = signal(Option::<User>::None);
    let (session, set_session) = signal(Option::<Session>::None);
    
    // Load existing session from LocalStorage on mount
    Effect::new(move |_| {
        if let Ok(stored_session) = LocalStorage::get::<Session>("sb_session") {
            set_user.set(Some(stored_session.user.clone()));
            set_session.set(Some(stored_session));
        }
    });

    provide_context(AuthContext { user, session, set_user, set_session });
    
    children()
}

#[derive(Copy, Clone)]
pub struct AuthContext {
    pub user: ReadSignal<Option<User>>,
    pub session: ReadSignal<Option<Session>>,
    pub set_user: WriteSignal<Option<User>>,
    pub set_session: WriteSignal<Option<Session>>,
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

    pub fn logout(&self) {
        self.set_user.set(None);
        self.set_session.set(None);
        LocalStorage::delete("sb_session");
    }
}

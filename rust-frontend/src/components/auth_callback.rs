use leptos::prelude::*;
use crate::auth::use_auth;
use gloo_storage::Storage;

#[component]
pub fn AuthCallback() -> impl IntoView {
    let auth = use_auth();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        let window = web_sys::window().unwrap();
        let hash = window.location().hash().unwrap_or_default();
        
        if !hash.is_empty() {
            let params = web_sys::UrlSearchParams::new_with_str(&hash.trim_start_matches('#')).unwrap();
            
            if let (Some(access_token), Some(token_type)) = (params.get("access_token"), params.get("type")) {
                // Extract payload to get user info (email, sub/id)
                let parts: Vec<&str> = access_token.split('.').collect();
                if parts.len() == 3 {
                    if let Ok(payload) = crate::auth::decode_base64_json(parts[1]) {
                        let user_id = payload.get("sub").and_then(|v| v.as_str());
                        let email = payload.get("email").and_then(|v| v.as_str());
                        
                        if let (Some(id), Some(email_addr)) = (user_id, email) {
                            let session = crate::auth::Session {
                                access_token: access_token.clone(),
                                user: crate::auth::User {
                                    id: id.to_string(),
                                    email: Some(email_addr.to_string()),
                                }
                            };
                            
                            let nav_async = navigate.clone();
                            leptos::task::spawn_local(async move {
                                // Verify token with backend before trusting it
                                if let Ok(_) = crate::api::ApiClient::get_balance(Some(&access_token)).await {
                                    // PERSIST: Set session in state and localStorage
                                    auth.set_session.set(Some(session.clone()));
                                    auth.set_user.set(Some(session.user.clone()));
                                    let _ = gloo_storage::LocalStorage::set("sb_session", session);
                                    
                                    // PROCEED: Handle actual flow based on type
                                    if token_type == "recovery" {
                                        nav_async("/reset-password", Default::default());
                                    } else {
                                        nav_async("/", Default::default());
                                    }
                                } else {
                                    leptos::logging::error!("AuthCallback: Token verification failed.");
                                    nav_async("/login", Default::default());
                                }
                            });
                            return;
                        }
                    }
                }
                // Fallback if decoding fails
                leptos::logging::error!("AuthCallback: Failed to decode security token payload.");
                navigate.clone()("/login", Default::default());
            }
        } else {
            navigate.clone()("/", Default::default());
        }
    });

    view! {
        <div class="fade-in" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 60vh; gap: var(--s-6);">
            <crate::components::icons::LoadingSpinner />
            <span class="scanning-text" style="font-weight: 800; letter-spacing: 0.1em;">"AUTHENTICATING INFRASTRUCTURE..."</span>
        </div>
    }
}

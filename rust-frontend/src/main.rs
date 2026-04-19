mod api;
mod auth;
mod components;

use leptos::prelude::*;
use leptos::either::Either;
use leptos_router::components::*;
use leptos_router::path;
use crate::auth::{AuthProvider, use_auth};

#[component]
fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                <div class="app-wrapper">
                    <header class="glass">
                        <div class="logo">
                            <span class="text-gradient">"PRECISION UPSCALE"</span>
                        </div>
                        <nav>
                            <a href="/">"Workspace"</a>
                            <a href="/history">"Archive"</a>
                            <AuthNav />
                        </nav>
                    </header>

                    <main>
                        <Routes fallback=|| view! { "Page not found." }>
                            <Route path=path!("/") view=Home />
                            <Route path=path!("/login") view=Login />
                            <Route path=path!("/history") view=History />
                        </Routes>
                    </main>
                </div>
            </Router>
        </AuthProvider>
    }
}

#[component]
fn AuthNav() -> impl IntoView {
    let auth = use_auth();
    
    move || match auth.user.get() {
        Some(user) => Either::Left(view! {
            <span class="user-badge">{user.email.clone().unwrap_or_default()}</span>
            <button class="btn btn-secondary btn-sm" on:click=move |_| auth.logout()>"Logout"</button>
        }),
        None => Either::Right(view! {
            <a href="/login" class="btn btn-primary btn-sm">"Login"</a>
        }),
    }
}

#[component]
fn Home() -> impl IntoView {
    let auth = use_auth();

    view! {
        <div class="fade-in">
            {move || match auth.user.get() {
                Some(_) => Either::Left(view! {
                    <crate::components::upload_zone::Dashboard />
                }),
                None => Either::Right(view! {
                    <div class="hero-section">
                        <h1 class="text-gradient">"Enterprise Image Enhancement"</h1>
                        <p class="text-muted" style="max-width: 600px; margin: 0 auto 2.5rem;">
                            "Professional-grade scaling powered by state-of-the-art neural reconstruction. 
                            Secure, precise, and optimized for high-fidelity workflows."
                        </p>
                        <div class="hero-actions">
                            <a href="/login" class="btn btn-primary" style="padding: 1rem 2.5rem;">"Get Started"</a>
                        </div>
                    </div>
                    
                    <div class="stats-grid">
                        <div class="stat-card">
                            <span class="stat-label">"Accuracy"</span>
                            <span class="stat-value">"99.9%"</span>
                        </div>
                        <div class="stat-card">
                            <span class="stat-label">"Processing"</span>
                            <span class="stat-value">"< 2s"</span>
                        </div>
                        <div class="stat-card">
                            <span class="stat-label">"Encryption"</span>
                            <span class="stat-value">"AES-256"</span>
                        </div>
                    </div>
                })
            }}
        </div>
    }
}

#[component]
fn Login() -> impl IntoView {
    let auth = use_auth();
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error_msg.set(None);
        
        let email_val = email.get();
        let pass_val = password.get();
        
        spawn_local(async move {
            match auth.login(&email_val, &pass_val).await {
                Ok(_) => {
                    let window = web_sys::window().unwrap();
                    let _ = window.location().set_href("/");
                }
                Err(e) => {
                    set_error_msg.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="fade-in auth-container">
            <div class="card auth-card">
                <h2>"Identity Provider"</h2>
                <p class="text-muted" style="margin-bottom: 2rem; font-size: 0.875rem;">"Sign in to access your secure workspace."</p>
                <form on:submit=on_submit>
                    <div class="input-group">
                        <label>"Corporate Email"</label>
                        <input 
                            type="email" 
                            placeholder="user@organization.com"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=email
                        />
                    </div>
                    <div class="input-group" style="margin-top: 1.25rem;">
                        <label>"Password"</label>
                        <input 
                            type="password" 
                            placeholder="••••••••"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                        />
                    </div>
                    
                    {move || match error_msg.get() {
                        Some(msg) => Either::Left(view! {
                            <p class="error-text" style="margin-top: 1rem;">{msg}</p>
                        }),
                        None => Either::Right(())
                    }}

                    <button 
                        type="submit" 
                        class="btn btn-primary" 
                        style="margin-top: 2rem; width: 100%;"
                        disabled=loading
                    >
                        {move || if loading.get() { "Authenticating..." } else { "Sign In" }}
                    </button>
                </form>
            </div>
        </div>
    }
}

#[component]
fn History() -> impl IntoView {
    view! {
        <div class="fade-in">
            <div class="page-header">
                <h2>"Asset Archive"</h2>
                <p class="text-muted">"Manage and download your previously processed images."</p>
            </div>
            <div class="card" style="text-align: center; padding: 4rem;">
                "Your archive is currently empty."
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    mount_to_body(App);
}

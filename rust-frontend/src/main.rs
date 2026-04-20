mod api;
mod auth;
mod components;

use leptos::prelude::*;
use leptos::either::Either;
use leptos_router::components::*;
use leptos_router::path;
use crate::auth::{AuthProvider, use_auth};
use crate::api::ApiClient;
use crate::components::icons::{Zap, HistoryIcon, LogOut, CreditCard};
use crate::components::auth::{Login, Register, ForgotPassword};
use crate::components::comparison_slider::ComparisonSlider;
use crate::components::configure::Configure;
use crate::components::view_result::ViewResult;

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub temp_file: ReadSignal<Option<web_sys::File>>,
    pub set_temp_file: WriteSignal<Option<web_sys::File>>,
    pub temp_classification: ReadSignal<Option<String>>,
    pub set_temp_classification: WriteSignal<Option<String>>,
}

pub fn provide_global_state() {
    let (temp_file, set_temp_file) = signal(None);
    let (temp_classification, set_temp_classification) = signal(None);
    provide_context(GlobalState { 
        temp_file, 
        set_temp_file, 
        temp_classification, 
        set_temp_classification 
    });
}

pub fn use_global_state() -> GlobalState {
    use_context::<GlobalState>().expect("GlobalState must be provided")
}

#[component]
fn AuthGuard(children: Children) -> impl IntoView {
    let auth = use_auth();
    let navigate = leptos_router::hooks::use_navigate();
    
    Effect::new(move |_| {
        if auth.user.get().is_none() {
            navigate("/", Default::default());
        }
    });

    children()
}

#[component]
fn App() -> impl IntoView {
    provide_global_state();

    view! {
        <AuthProvider>
            <Router>
                <div class="app-wrapper">
                    <header class="glass">
                        <div class="logo">
                            <div class="logo-icon"><Zap size={18} /></div>
                            "PRECISION" 
                            <span style="opacity: 0.5;">"UPSCALE"</span>
                        </div>
                        <nav>
                            <a href="/">"UPSCALE"</a>
                            <a href="/history">"HISTORY"</a>
                            <a href="/settings">"CREDITS"</a>
                            <AuthNav />
                        </nav>
                    </header>

                    <main>
                        <Routes fallback=|| view! { <NotFound /> }>
                            <Route path=path!("/") view=Home />
                            <Route path=path!("/login") view=Login />
                            <Route path=path!("/register") view=Register />
                            <Route path=path!("/forgot-password") view=ForgotPassword />
                            
                            <Route path=path!("/configure") view=|| view! { <AuthGuard><Configure /></AuthGuard> } />
                            <Route path=path!("/view/:job_id") view=|| view! { <AuthGuard><ViewResult /></AuthGuard> } />
                            <Route path=path!("/history") view=|| view! { <AuthGuard><History /></AuthGuard> } />
                            <Route path=path!("/settings") view=|| view! { <AuthGuard><Credits /></AuthGuard> } />
                        </Routes>
                    </main>

                    <Footer />
                </div>
            </Router>
        </AuthProvider>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer>
            <div class="footer-content">
                <div class="footer-logo">
                    <Zap size={14} />
                    "PRECISION UPSCALE"
                </div>
                <div class="footer-links">
                    <span>"© 2026 INFRASTRUCTURE"</span>
                    <span class="divider">"|"</span>
                    <span>"V7.1 STABLE"</span>
                </div>
            </div>
            <style>
                "footer { border-top: 1px solid var(--border-color); padding: 2rem 4rem; margin-top: auto; background: var(--bg-color); }
                .footer-content { display: flex; justify-content: space-between; align-items: center; max-width: 1200px; margin: 0 auto; width: 100%; }
                .footer-logo { font-size: 0.75rem; font-weight: 700; color: var(--text-muted); display: flex; align-items: center; gap: 0.5rem; letter-spacing: 0.05em; }
                .footer-links { font-size: 0.7rem; color: var(--text-muted); font-weight: 600; display: flex; gap: 1rem; align-items: center; }
                .divider { opacity: 0.2; }
                "
            </style>
        </footer>
    }
}

#[component]
fn AuthNav() -> impl IntoView {
    let auth = use_auth();
    
    let balance = LocalResource::new(
        move || {
            let token = auth.session.get().map(|s| s.access_token);
            async move {
                ApiClient::get_balance(token.as_deref()).await
            }
        }
    );
    
    let (show_dropdown, set_show_dropdown) = signal(false);
    
    view! {
        {move || match auth.user.get() {
            Some(user) => Either::Left(view! {
                <div style="display: flex; align-items: center; gap: 1.25rem;">
                    <Suspense>
                        {move || Suspend::new(async move {
                            let res = balance.await;
                            match res {
                                Ok(credits) => view! { 
                                    <div class="balance-pill">
                                        <Zap size={12} />
                                        <strong>{credits}</strong>
                                        <span>"CREDITS"</span>
                                    </div>
                                }.into_any(),
                                _ => ().into_any(),
                            }
                        })}
                    </Suspense>
                    
                    <div class="dropdown-container">
                        <div 
                            class="avatar-btn"
                            on:mouseenter=move |_| set_show_dropdown.set(true)
                            on:click=move |ev| {
                                ev.stop_propagation();
                                set_show_dropdown.update(|v| *v = !*v);
                            }
                        >
                            {user.email.clone().unwrap_or_default().chars().next().unwrap_or('?').to_uppercase().to_string()}
                        </div>
                        <div 
                            class="dropdown-menu"
                            class:show=show_dropdown
                            on:mouseleave=move |_| set_show_dropdown.set(false)
                            on:click=move |ev| {
                                ev.stop_propagation();
                                set_show_dropdown.set(false);
                            }
                        >
                            <div class="dropdown-header">
                                <span class="user-email">{user.email.clone().unwrap_or_default()}</span>
                            </div>
                            <a href="/history" class="dropdown-item">
                                <HistoryIcon size={16} />
                                "My History"
                            </a>
                            <a href="/settings" class="dropdown-item">
                                <CreditCard size={16} />
                                "Billing & Credits"
                            </a>
                            <div class="dropdown-divider"></div>
                            <div class="dropdown-item error" on:click=move |_| auth.logout()>
                                <LogOut size={16} />
                                "Sign Out"
                            </div>
                        </div>
                    </div>
                </div>
            }),
            None => Either::Right(view! {
                <div style="display: flex; gap: 0.75rem;">
                    <a href="/login" class="btn btn-secondary btn-sm">"Sign In"</a>
                    <a href="/register" class="btn btn-primary btn-sm">"Create Account"</a>
                </div>
            }),
        }}
    }
}

#[component]
fn Credits() -> impl IntoView {
    view! {
        <crate::components::settings::Credits />
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="fade-in">
            <div class="hero-section">
                <h1 class="text-gradient">"Professional Super-Resolution"</h1>
                <p class="muted" style="max-width: 600px; margin: 0 auto 1rem; font-size: 1.125rem;">
                    "Bespoke neural upscaling for photography and illustration."
                </p>
                <p class="muted" style="max-width: 600px; margin: 0 auto 3rem; font-size: 0.875rem; opacity: 0.7;">
                    "Restore frequency details, eliminate compression, and reach target resolutions with surgical precision."
                </p>
                
                <div class="hybrid-layout">
                    <div class="hybrid-left">
                        <ComparisonSlider 
                            before_url="/assets/hero_before.png".to_string() 
                            after_url="/assets/hero_after.png".to_string() 
                            before_label="Original (Compressed)"
                            after_label="Upscaled (2K/4K)"
                        />
                    </div>
                    <div class="hybrid-right">
                        <div class="card hybrid-card">
                            <crate::components::upload_zone::UploadZone />
                        </div>
                        
                        <div class="hybrid-stats">
                            <div class="h-stat">
                                <span class="h-label">"Model"</span>
                                <span class="h-value">"V7.1 STABLE"</span>
                            </div>
                            <div class="h-stat">
                                <span class="h-label">"Max Output"</span>
                                <span class="h-value">"4K UHD"</span>
                            </div>
                        </div>
                    </div>
                </div>

                <style>
                    ".hybrid-layout { 
                        display: grid; 
                        grid-template-columns: 1.2fr 1fr; 
                        gap: 2rem; 
                        margin-top: 2rem; 
                        text-align: left;
                        align-items: flex-start;
                    }
                    .hybrid-card { padding: 1rem; height: 100%; display: flex; flex-direction: column; }
                    .hybrid-stats { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1.5rem; }
                    .h-stat { background: var(--bg-color); border: 1px solid var(--border-color); padding: 1rem; border-radius: 8px; }
                    .h-label { display: block; font-size: 0.6rem; color: var(--text-muted); font-weight: 800; letter-spacing: 0.05em; text-transform: uppercase; margin-bottom: 0.25rem; }
                    .h-value { font-size: 0.9rem; font-weight: 700; color: var(--text-color); font-family: var(--font-mono); }
                    
                    @media (max-width: 1000px) {
                        .hybrid-layout { grid-template-columns: 1fr; }
                    }
                    "
                </style>
            </div>
        </div>
    }
}

#[component]
fn History() -> impl IntoView {
    view! {
        <crate::components::history_gallery::HistoryGallery />
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="fade-in" style="text-align: center; padding: 10rem 0;">
            <h1 style="font-size: 5rem; font-weight: 800; opacity: 0.1;">"404"</h1>
            <h2 style="margin-top: -2rem;">"Resource Not Found"</h2>
            <p class="muted" style="margin-top: 1rem; margin-bottom: 3rem;">"The requested coordinate does not exist in our infrastructure."</p>
            <a href="/" class="btn btn-primary">"Return Home"</a>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    mount_to_body(App);
}

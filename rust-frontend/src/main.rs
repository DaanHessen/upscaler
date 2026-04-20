mod api;
mod auth;
mod components;
mod persistence;

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
use crate::components::legal::{Terms, Contact};

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub temp_file: ReadSignal<Option<web_sys::File>>,
    pub set_temp_file: WriteSignal<Option<web_sys::File>>,
    pub temp_classification: ReadSignal<Option<String>>,
    pub set_temp_classification: WriteSignal<Option<String>>,
    // Persistent settings
    pub quality: ReadSignal<String>,
    pub set_quality: WriteSignal<String>,
    pub style: ReadSignal<String>,
    pub set_style: WriteSignal<String>,
    pub temperature: ReadSignal<f32>,
    pub set_temperature: WriteSignal<f32>,
    pub keep_aspect_ratio: ReadSignal<bool>,
    pub set_keep_aspect_ratio: WriteSignal<bool>,
    pub keep_depth_of_field: ReadSignal<bool>,
    pub set_keep_depth_of_field: WriteSignal<bool>,
    pub lighting: ReadSignal<String>,
    pub set_lighting: WriteSignal<String>,
    pub preview_base64: ReadSignal<Option<String>>,
    pub set_preview_base64: WriteSignal<Option<String>>,
    pub thinking_level: ReadSignal<String>,
    pub set_thinking_level: WriteSignal<String>,
}

pub fn provide_global_state() {
    let (temp_file, set_temp_file) = signal(None);
    let (temp_classification, set_temp_classification) = signal(None);
    let initial_settings = crate::persistence::load_settings();
    
    let (quality, set_quality) = signal(initial_settings.as_ref().map(|s| s.quality.clone()).unwrap_or_else(|| "2K".to_string()));
    let (style, set_style) = signal("PHOTOGRAPHY".to_string());
    let (temperature, set_temperature) = signal(initial_settings.as_ref().map(|s| s.temperature).unwrap_or(0.0f32));
    let (keep_aspect_ratio, set_keep_aspect_ratio) = signal(initial_settings.as_ref().map(|s| s.keep_aspect_ratio).unwrap_or(true));
    let (keep_depth_of_field, set_keep_depth_of_field) = signal(initial_settings.as_ref().map(|s| s.keep_depth_of_field).unwrap_or(true));
    let (lighting, set_lighting) = signal(initial_settings.as_ref().map(|s| s.lighting.clone()).unwrap_or_else(|| "Original".to_string()));
    let (thinking_level, set_thinking_level) = signal(initial_settings.as_ref().map(|s| s.thinking_level.clone()).unwrap_or_else(|| "HIGH".to_string()));
    let (preview_base64, set_preview_base64) = signal(None);
    
    provide_context(GlobalState { 
        temp_file, 
        set_temp_file, 
        temp_classification, 
        set_temp_classification,
        quality,
        set_quality,
        style,
        set_style,
        temperature,
        set_temperature,
        keep_aspect_ratio,
        set_keep_aspect_ratio,
        keep_depth_of_field,
        set_keep_depth_of_field,
        lighting,
        set_lighting,
        preview_base64,
        set_preview_base64,
        thinking_level,
        set_thinking_level,
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

    let _auth_ctx = use_context::<crate::auth::AuthContext>().unwrap_or_else(|| {
        // Fallback if needed, but AuthNav will handle it
        // ...
        unreachable!("AuthContext should be provided by AuthProvider wrap")
    });

    // 1. Hydroate Global State from Storage
    let global_state = use_global_state();
    
    // Sync classification changes to storage
    Effect::new(move |_| {
        persistence::save_classification(global_state.temp_classification.get());
    });

    // Sync settings changes to storage
    Effect::new(move |_| {
        persistence::save_settings(persistence::SettingsState {
            quality: global_state.quality.get(),
            style: global_state.style.get(),
            temperature: global_state.temperature.get(),
            keep_aspect_ratio: global_state.keep_aspect_ratio.get(),
            keep_depth_of_field: global_state.keep_depth_of_field.get(),
            lighting: global_state.lighting.get(),
            thinking_level: global_state.thinking_level.get(),
        });
    });

    // Hydrate everything on start
    Effect::new(move |_| {
        let gs = global_state;
        
        // Hydrate classification (sync)
        if let Some(c) = persistence::load_classification() {
            gs.set_temp_classification.set(Some(c));
        }

        // Hydrate settings (sync)
        if let Some(s) = persistence::load_settings() {
            gs.set_quality.set(s.quality);
            gs.set_style.set(s.style);
            gs.set_temperature.set(s.temperature);
            gs.set_keep_aspect_ratio.set(s.keep_aspect_ratio);
            gs.set_keep_depth_of_field.set(s.keep_depth_of_field);
            gs.set_lighting.set(s.lighting);
        }

        // Hydrate file (async)
        leptos::task::spawn_local(async move {
            if let Some(f) = persistence::load_file().await {
                gs.set_temp_file.set(Some(f));
            }
        });
    });

    view! {
        <Router>
            <MainLayout />
        </Router>
    }
}

pub fn Root() -> impl IntoView {
    view! {
        <AuthProvider>
            <App />
        </AuthProvider>
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    let auth = use_auth();
    
    view! {
        <div class="app-wrapper">
            <header class="glass stagger-1">
                <a href="/" class="logo" style="text-decoration: none; display: flex; align-items: center; gap: var(--s-3); color: inherit;">
                    <div class="logo-icon"><Zap size={18} /></div>
                    "UPSYL" 
                    <span>"STUDIO"</span>
                </a>
                <nav>
                    <a href="/">"STUDIO"</a>
                    {move || auth.user.get().is_some().then(|| view! {
                        <>
                            <a href="/history">"HISTORY"</a>
                            <a href="/settings">"CREDITS"</a>
                        </>
                    })}
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
                    <Route path=path!("/terms") view=Terms />
                    <Route path=path!("/contact") view=Contact />
                </Routes>
            </main>

            <Footer />
        </div>
    }
}

#[component]
fn Footer() -> impl IntoView {
    let health = LocalResource::new(move || async move {
        ApiClient::get_health().await.unwrap_or(false)
    });

    view! {
        <footer>
            <div class="footer-content">
                <div class="footer-left">
                    <div class="footer-logo">
                        <Zap size={14} />
                        "UPSYL STUDIO"
                    </div>
                </div>
                
                <div class="footer-center">
                    <span class="footer-meta">"© 2026 UPSYL"</span>
                    <span class="divider">"|"</span>
                    <a href="/terms" class="footer-link">"Terms"</a>
                    <span class="divider">"|"</span>
                    <a href="/contact" class="footer-link">"Support"</a>
                </div>

                <div class="footer-right">
                    <span class="footer-meta">"SYSTEM:"</span>
                    <Suspense fallback=|| view! { <span class="footer-status-tag">"CHECKING..."</span> }>
                        {move || match health.get() {
                            Some(h) if *h => view! { <span class="status-indicator online"></span><span class="footer-status-tag online">"ONLINE"</span> }.into_any(),
                            _ => view! { <span class="status-indicator offline"></span><span class="footer-status-tag offline">"OFFLINE"</span> }.into_any(),
                        }}
                    </Suspense>
                </div>
            </div>
            <style>
                "footer { border-top: 1px solid var(--glass-border); padding: var(--s-6) var(--s-12); margin-top: auto; background: hsl(var(--bg) / 0.8); backdrop-filter: blur(20px); }
                .footer-content { display: flex; justify-content: space-between; align-items: center; max-width: 1200px; margin: 0 auto; width: 100%; height: var(--s-12); }
                
                .footer-left { flex: 1; display: flex; align-items: center; }
                .footer-logo { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text)); display: flex; align-items: center; gap: var(--s-2); letter-spacing: 0.2em; text-transform: uppercase; }
                
                .hero-content { max-width: 800px; margin: 0 auto 4rem; text-align: center; }
                .hero-subtitle { font-size: 1.25rem; font-weight: 800; color: hsl(var(--text)); margin-bottom: var(--s-4); letter-spacing: -0.02em; }
                .hero-description { font-size: 1.0625rem; font-weight: 500; color: hsl(var(--text-muted)); line-height: 1.6; }

                .footer-center { flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--s-4); }
                .footer-meta { font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; }
                .footer-link { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-decoration: none; text-transform: uppercase; letter-spacing: 0.1em; transition: color 0.2s; }
                .footer-link:hover { color: hsl(var(--accent)); }
                .divider { opacity: 0.1; color: hsl(var(--text-dim)); }

                .footer-right { flex: 1; display: flex; align-items: center; justify-content: flex-end; gap: var(--s-3); }
                .footer-status-tag { font-size: 0.625rem; font-weight: 850; letter-spacing: 0.1em; }
                .footer-status-tag.online { color: hsl(var(--success)); }
                .footer-status-tag.offline { color: hsl(var(--error)); }

                .status-indicator { width: 6px; height: 6px; border-radius: 50%; display: inline-block; }
                .status-indicator.online { background: hsl(var(--success)); box-shadow: 0 0 8px hsl(var(--success) / 0.8); animation: pulse 2s infinite; }
                .status-indicator.offline { background: hsl(var(--error)); }

                @keyframes pulse {
                    0% { transform: scale(0.95); box-shadow: 0 0 0 0 hsl(var(--success) / 0.7); }
                    70% { transform: scale(1); box-shadow: 0 0 0 6px hsl(var(--success) / 0); }
                    100% { transform: scale(0.95); box-shadow: 0 0 0 0 hsl(var(--success) / 0); }
                }
                
                @media (max-width: 768px) {
                    footer { padding: var(--s-8) var(--s-6); }
                    .footer-content { flex-direction: column; height: auto; gap: var(--s-6); }
                    .footer-left, .footer-center, .footer-right { flex: none; justify-content: center; }
                }
                "
            </style>
        </footer>
    }
}

#[component]
fn AuthNav() -> impl IntoView {
    let auth = use_auth();
    
    // Trigger throttled telemetry sync on mount
    Effect::new(move |_| {
        if auth.user.get().is_some() {
            auth.sync_telemetry(false);
        }
    });

    let (show_dropdown, set_show_dropdown) = signal(false);
    
    view! {
        {move || match auth.user.get() {
            Some(user) => Either::Left(view! {
                    <div style="display: flex; align-items: center; gap: var(--s-6);">
                        <Suspense>
                            {move || {
                                let _history = auth.history;
                                let res = auth.credits.get();
                                match res {
                                    Some(credits) => view! { 
                                        <div class="balance-pill">
                                            <Zap size={12} />
                                            <strong>{credits}</strong>
                                            <span>"UNITS"</span>
                                        </div>
                                    }.into_any(),
                                    _ => ().into_any(),
                                }
                            }}
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
                <h1 class="text-gradient stagger-1">"Professional Super-Resolution"</h1>
                <div class="hero-content stagger-2">
                    <h2 class="hero-subtitle">"Professional AI upscaling for photographers and creators."</h2>
                    <p class="hero-description">"Sharpen details, remove artifacts, and upscale to 4K with surgical precision."</p>
                </div>
                
                <div class="hybrid-layout stagger-3">
                    <div class="studio-card hybrid-left">
                        <ComparisonSlider 
                            before_url="assets/hero_before.svg".to_string() 
                            after_url="assets/hero_after.svg".to_string() 
                            before_label="BEFORE (ORIGINAL)"
                            after_label="AFTER (UPSCALED)"
                        />
                    </div>
                    <div class="studio-card hybrid-right">
                        <crate::components::upload_zone::UploadZone />
                    </div>
                </div>

            <style>
                ".hero-section { padding: var(--s-10) 0 var(--s-20); }                .hybrid-layout { 
                    display: grid; 
                    grid-template-columns: 1fr 1fr; 
                    gap: var(--s-16); 
                    margin-top: var(--s-16); 
                    text-align: left;
                    align-items: stretch;
                    max-width: 1300px;
                    margin-left: auto;
                    margin-right: auto;
                }
                
                .studio-card { 
                    background: hsl(var(--surface));
                    border: 1px solid hsl(var(--accent) / 0.1);
                    border-radius: var(--radius-lg);
                    box-shadow: 0 40px 100px -30px rgba(0,0,0,0.8);
                    overflow: hidden;
                    position: relative;
                }

                .hybrid-right { padding: var(--s-10); display: flex; flex-direction: column; justify-content: center; }
              .h-stat:hover { border-color: hsl(var(--accent) / 0.3); }
                .h-label { display: block; font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 800; letter-spacing: 0.1em; text-transform: uppercase; margin-bottom: var(--s-1); }
                .h-value { font-size: 0.8125rem; font-weight: 700; color: hsl(var(--text)); font-family: var(--font-mono); }
                
                @media (max-width: 1050px) {
                    .hero-section { padding: var(--s-6) 0 var(--s-12); }
                    .hybrid-layout { 
                        grid-template-columns: 1fr; 
                        max-width: 600px; 
                        gap: var(--s-8);
                    }
                    .hybrid-left { order: 2; }
                    .hybrid-right { order: 1; }
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
    mount_to_body(Root);
}

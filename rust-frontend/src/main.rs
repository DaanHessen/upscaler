mod api;
mod auth;
mod components;
mod persistence;
mod text;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use crate::auth::{AuthProvider, use_auth};
use crate::api::{ApiClient, PollResponse};
use leptos_router::hooks::use_location;
use crate::components::icons::{Zap, LogOut, Sun, Moon, UserIcon, Target, RefreshCw, ImageIcon, NovuraLogo};
use crate::components::auth::{Login, Register, ForgotPassword};
use crate::components::comparison_slider::ComparisonSlider;
use crate::components::configure::Configure;
use crate::components::view_result::ViewResult;
use crate::components::legal::{Terms, Contact, Privacy, AUP, CookiePolicy};
use crate::components::auth_callback::AuthCallback;
use crate::components::reset_password::ResetPassword;
use leptos::ev;
use leptos_router::hooks::use_navigate;
use crate::components::profile::AccountSettings;
use crate::components::admin::AdminPanel;
use crate::components::cookie_banner::CookieBanner;

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub quality: ReadSignal<String>,
    pub set_quality: WriteSignal<String>,
    pub style: ReadSignal<String>,
    pub set_style: WriteSignal<String>,
    pub temperature: ReadSignal<f32>,
    pub set_temperature: WriteSignal<f32>,
    pub keep_depth_of_field: ReadSignal<bool>,
    pub set_keep_depth_of_field: WriteSignal<bool>,
    pub lighting: ReadSignal<String>,
    pub set_lighting: WriteSignal<String>,
    pub thinking_level: ReadSignal<String>,
    pub set_thinking_level: WriteSignal<String>,
    pub seed: ReadSignal<Option<u32>>,
    pub set_seed: WriteSignal<Option<u32>>,
    pub temp_file: ReadSignal<Option<web_sys::File>>,
    pub set_temp_file: WriteSignal<Option<web_sys::File>>,
    pub temp_classification: ReadSignal<Option<String>>,
    pub set_temp_classification: WriteSignal<Option<String>>,
    pub preview_base64: ReadSignal<Option<String>>,
    pub set_preview_base64: WriteSignal<Option<String>>,
    pub theme: ReadSignal<String>,
    pub set_theme: WriteSignal<String>,
    pub active_tool: ReadSignal<String>,
    pub set_active_tool: WriteSignal<String>,
    pub target_medium: ReadSignal<String>,
    pub set_target_medium: WriteSignal<String>,
    pub render_style: ReadSignal<String>,
    pub set_render_style: WriteSignal<String>,
    pub target_aspect_ratio: ReadSignal<String>,
    pub set_target_aspect_ratio: WriteSignal<String>,
    pub notification: ReadSignal<Option<(String, String)>>,
    pub set_notification: WriteSignal<Option<(String, String)>>,
    pub processing_job: ReadSignal<Option<uuid::Uuid>>,
    pub set_processing_job: WriteSignal<Option<uuid::Uuid>>,
    pub engine_status: ReadSignal<Option<crate::api::PollResponse>>,
    pub set_engine_status: WriteSignal<Option<crate::api::PollResponse>>,
    pub is_submitting: ReadSignal<bool>,
    pub set_is_submitting: WriteSignal<bool>,
    pub avg_latency_secs: ReadSignal<i32>,
    pub set_avg_latency_secs: WriteSignal<i32>,
    pub pre_processing: ReadSignal<String>,
    pub set_pre_processing: WriteSignal<String>,
    pub post_polish: ReadSignal<String>,
    pub set_post_polish: WriteSignal<String>,
    pub debug_gemini_only: ReadSignal<bool>,
    pub set_debug_gemini_only: WriteSignal<bool>,
    pub topaz_mode: ReadSignal<String>,
    pub set_topaz_mode: WriteSignal<String>,
}

impl GlobalState {
    pub fn show_error(&self, msg: impl Into<String>) {
        self.set_notification.set(Some((msg.into(), "error".to_string())));
    }
    pub fn show_success(&self, msg: impl Into<String>) {
        self.set_notification.set(Some((msg.into(), "success".to_string())));
    }
    pub fn clear_notification(&self) {
        self.set_notification.set(None);
    }
}

pub fn provide_global_state() {
    let (quality, set_quality) = signal("2K".to_string());
    let (style, set_style) = signal("PHOTOGRAPHY".to_string());
    let (temperature, set_temperature) = signal(0.0);
    let (keep_depth_of_field, set_keep_depth_of_field) = signal(false);
    let (lighting, set_lighting) = signal("Original".to_string());
    let (thinking_level, set_thinking_level) = signal("MINIMAL".to_string());
    let (seed, set_seed) = signal(None::<u32>);
    let (temp_file, set_temp_file) = signal(None::<web_sys::File>);
    let (temp_classification, set_temp_classification) = signal(None::<String>);
    let (preview_base64, set_preview_base64) = signal(None::<String>);
    let (theme, set_theme) = signal("dark".to_string());
    let (active_tool, set_active_tool) = signal("UPSCALE".to_string());
    let (target_medium, set_target_medium) = signal("3D Render".to_string());
    let (render_style, set_render_style) = signal("Photorealistic".to_string());
    let (target_aspect_ratio, set_target_aspect_ratio) = signal("16:9".to_string());
    let (notification, set_notification) = signal(Option::<(String, String)>::None);
    let (processing_job, set_processing_job) = signal(Option::<uuid::Uuid>::None);
    let (engine_status, set_engine_status) = signal(Option::<PollResponse>::None);
    let (is_submitting, set_is_submitting) = signal(false);
    let (avg_latency_secs, set_avg_latency_secs) = signal(20);
    let (pre_processing, set_pre_processing) = signal("Off".to_string());
    let (post_polish, set_post_polish) = signal("Off".to_string());
    let (debug_gemini_only, set_debug_gemini_only) = signal(false);
    let (topaz_mode, set_topaz_mode) = signal("Auto".to_string());

    provide_context(GlobalState {
        quality, set_quality,
        style, set_style,
        temperature, set_temperature,
        keep_depth_of_field, set_keep_depth_of_field,
        lighting, set_lighting,
        thinking_level, set_thinking_level,
        seed, set_seed,
        temp_file, set_temp_file,
        temp_classification, set_temp_classification,
        preview_base64, set_preview_base64,
        theme, set_theme,
        active_tool, set_active_tool,
        target_medium, set_target_medium,
        render_style, set_render_style,
        target_aspect_ratio, set_target_aspect_ratio,
        notification, set_notification,
        processing_job, set_processing_job,
        engine_status, set_engine_status,
        is_submitting, set_is_submitting,
        avg_latency_secs, set_avg_latency_secs,
        pre_processing, set_pre_processing,
        post_polish, set_post_polish,
        debug_gemini_only, set_debug_gemini_only,
        topaz_mode, set_topaz_mode,
    });
}

pub fn use_global_state() -> GlobalState {
    use_context::<GlobalState>().expect("GlobalState must be provided")
}

#[component]
fn App() -> impl IntoView {
    let gs = use_global_state();
    
    // Effects for persisting state
    Effect::new(move |_| {
        let theme_val = gs.theme.get();
        let settings = persistence::UserSettings {
            quality: gs.quality.get(),
            style: gs.style.get(),
            temperature: gs.temperature.get(),
            keep_depth_of_field: gs.keep_depth_of_field.get(),
            lighting: gs.lighting.get(),
            thinking_level: gs.thinking_level.get(),
            seed: gs.seed.get(),
            theme: theme_val.clone(),
            active_tool: gs.active_tool.get(),
            target_medium: gs.target_medium.get(),
            render_style: gs.render_style.get(),
            target_aspect_ratio: gs.target_aspect_ratio.get(),
            pre_processing: gs.pre_processing.get(),
            post_polish: gs.post_polish.get(),
            debug_gemini_only: gs.debug_gemini_only.get(),
            topaz_mode: gs.topaz_mode.get(),
        };
        persistence::save_settings(&settings);
        
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", &theme_val);
            }
        }
    });

    // Initial hydration
    Effect::new(move |_| {
        if let Some(c) = persistence::load_classification() {
            gs.set_temp_classification.set(Some(c));
        }

        // Hydrate settings (sync)
        if let Some(s) = persistence::load_settings() {
            gs.set_quality.set(s.quality);
            gs.set_style.set(s.style);
            gs.set_temperature.set(s.temperature);
            gs.set_keep_depth_of_field.set(s.keep_depth_of_field);
            gs.set_lighting.set(s.lighting);
            gs.set_thinking_level.set(s.thinking_level);
            gs.set_seed.set(s.seed);
            gs.set_theme.set(s.theme);
            gs.set_active_tool.set(s.active_tool);
            gs.set_target_medium.set(s.target_medium);
            gs.set_render_style.set(s.render_style);
            gs.set_target_aspect_ratio.set(s.target_aspect_ratio);
            gs.set_pre_processing.set(s.pre_processing);
            gs.set_post_polish.set(s.post_polish);
            gs.set_debug_gemini_only.set(s.debug_gemini_only);
            gs.set_topaz_mode.set(s.topaz_mode);
        }

        // Hydrate file (async)
        leptos::task::spawn_local(async move {
            if let Some(f) = persistence::load_file().await {
                gs.set_temp_file.set(Some(f));
            }
        });
    });

    let auth = crate::auth::use_auth();
    
    // Global Polling Loop for Upscale Jobs
    Effect::new(move |_| {
        if let Some(job_id) = gs.processing_job.get() {
            let token = auth.session.get().map(|s| s.access_token);
            let state = gs;
            leptos::task::spawn_local(async move {
                loop {
                    // Check if job is still the active one
                    if untrack(move || state.processing_job.get()) != Some(job_id) { break; }
                    
                    match ApiClient::poll_job(job_id, token.as_deref()).await {
                        Ok(resp) => {
                            state.set_engine_status.set(Some(resp.clone()));
                            if resp.status == "COMPLETED" {
                                if let Some(url) = resp.image_url {
                                    state.set_preview_base64.set(Some(url));
                                }
                                state.set_processing_job.set(None);
                                break;
                            }
                            if resp.status == "FAILED" {
                                state.show_error(resp.error.unwrap_or_else(|| "Upscale failed.".to_string()));
                                state.set_processing_job.set(None);
                                break;
                            }
                        },
                        Err(_) => {
                            gloo_timers::future::TimeoutFuture::new(5000).await;
                        }
                    }
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }
            });
        }
    });

    // Global Avg Latency refresh on mount
    leptos::task::spawn_local(async move {
        if let Ok(ms) = ApiClient::get_avg_latency().await {
            gs.set_avg_latency_secs.set((ms / 1000).max(5));
        }
    });

    view! {
        <Router>
            <MainLayout />
        </Router>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn Root() -> impl IntoView {
    provide_global_state();
    view! {
        <AuthProvider>
            <App />
        </AuthProvider>
    }
}

#[component]
fn NavLink(href: &'static str, children: Children) -> impl IntoView {
    let location = use_location();
    let is_active = move || {
        let path = location.pathname.get();
        if href == "/" {
            path == "/"
        } else {
            path.starts_with(href)
        }
    };

    view! {
        <A 
            href=href 
            attr:class=move || if is_active() { "nav-link active" } else { "nav-link" }
        >
            {children()}
        </A>
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    let auth = use_auth();
    
    view! {
        <div class="app-wrapper" style="position: relative;">
            <div class="bg-glow-container">
                <div class="bg-glow"></div>
                <div class="bg-glow glow-2"></div>
            </div>
            <header class="glass">
                <A href="/" attr:class="logo" attr:style="text-decoration: none; display: flex; align-items: center; gap: var(--s-3); color: inherit;">
                    <div class="logo-icon"><NovuraLogo size={18} /></div>
                    {crate::text::TXT.brand_name}
                    <span>{crate::text::TXT.brand_suffix}</span>
                </A>
                <nav>
                    <NavLink href="/">{crate::text::TXT.nav_home}</NavLink>
                    <NavLink href="/editor">{crate::text::TXT.nav_editor}</NavLink>
                    {move || auth.user.get().is_some().then(|| view! {
                        <>
                            <NavLink href="/history">{crate::text::TXT.nav_history}</NavLink>
                            <NavLink href="/settings">{crate::text::TXT.nav_billing}</NavLink>
                        </>
                    })}
                </nav>
                <div class="auth-nav-container">
                    <AuthNav />
                </div>
            </header>

            <CookieBanner />
            <crate::components::notifications::NotificationOverlay />

            <main>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/register") view=Register />
                    <Route path=path!("/forgot-password") view=ForgotPassword />
                    <Route path=path!("/reset-password") view=ResetPassword />
                    <Route path=path!("/auth/callback") view=AuthCallback />
                    
                    <Route path=path!("/editor") view=|| view! { <AuthGuard><Configure /></AuthGuard> } />
                    <Route path=path!("/view/:job_id") view=|| view! { <AuthGuard><ViewResult /></AuthGuard> } />
                    <Route path=path!("/history") view=|| view! { <AuthGuard><History /></AuthGuard> } />
                    <Route path=path!("/settings") view=|| view! { <AuthGuard><Credits /></AuthGuard> } />
                    <Route path=path!("/account") view=|| view! { <AuthGuard><AccountSettings /></AuthGuard> } />
                    <Route path=path!("/admin") view=|| view! { <AuthGuard><AdminPanel /></AuthGuard> } />
                    <Route path=path!("/terms") view=Terms />
                    <Route path=path!("/privacy") view=Privacy />
                    <Route path=path!("/rules") view=AUP />
                    <Route path=path!("/cookies") view=CookiePolicy />
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
                    <A href="/" attr:class="logo">
                        <div class="logo-icon"><crate::components::icons::NovuraLogo size={24} /></div>
                        {crate::text::TXT.brand_name} <span>{crate::text::TXT.brand_suffix}</span>
                    </A>
                </div>
                
                <div class="footer-center">
                    <span class="footer-meta">{format!("© 2026 {}", crate::text::TXT.brand_name)}</span>
                    <span class="divider">"|"</span>
                    <A href="/terms" attr:class="footer-link">"Terms"</A>
                    <span class="divider">"•"</span>
                    <A href="/privacy" attr:class="footer-link">"Privacy"</A>
                    <span class="divider">"•"</span>
                    <A href="/rules" attr:class="footer-link">"Rules"</A>
                    <span class="divider">"•"</span>
                    <A href="/cookies" attr:class="footer-link">"Cookies"</A>
                    <span class="divider">"|"</span>
                    <A href="/contact" attr:class="footer-link">"Support"</A>
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
    let theme = use_global_state().theme;
    let set_theme = use_global_state().set_theme;
    
    view! {
        {move || match auth.user.get() {
            Some(user) => view! {
                    <div class="auth-cluster" style="display: flex; align-items: center; gap: var(--s-6);">
                        <Suspense>
                            {move || {
                                let res = auth.credits.get();
                                match res {
                                    Some(credits) => view! { 
                                        <div class="balance-pill">
                                            <Zap size={12} />
                                            <strong>{credits}</strong>
                                            <span>{crate::text::TXT.label_credits}</span>
                                        </div>
                                    }.into_any(),
                                    _ => ().into_any(),
                                }
                            }}
                        </Suspense>
                        
                        <div class="dropdown-container"
                            on:mouseenter=move |_| set_show_dropdown.set(true)
                            on:mouseleave=move |_| set_show_dropdown.set(false)
                        >
                            <div 
                                class="avatar-btn"
                                on:click=move |ev: ev::MouseEvent| {
                                    ev.stop_propagation();
                                    set_show_dropdown.update(|v| *v = !*v);
                                }
                            >
                                {user.email.as_ref().and_then(|e| e.chars().next()).unwrap_or('?').to_uppercase().to_string()}
                            </div>

                            <div class="dropdown-menu" class:show=show_dropdown>
                                <div class="dropdown-header">
                                    <span class="user-email">{user.email.clone().unwrap_or_default()}</span>
                                </div>
                                <div class="dropdown-divider"></div>
                                
                                <div class="dropdown-item" on:click=move |_| {
                                    let next = if theme.get() == "light" { "dark" } else { "light" };
                                    set_theme.set(next.to_string());
                                }>
                                    {move || if theme.get() == "light" { view! { <Moon size={16} /> }.into_any() } else { view! { <Sun size={16} /> }.into_any() }}
                                    {move || if theme.get() == "light" { "dark mode" } else { "light mode" }}
                                </div>
                                
                                <A href="/account" attr:class="dropdown-item" attr:style="text-decoration: none;">
                                    <UserIcon size={16} />
                                    "Profile Settings"
                                </A>
                                <div class="dropdown-divider"></div>
                                <A href="/terms" attr:class="dropdown-item" attr:style="text-decoration: none;">"Terms"</A>
                                <A href="/privacy" attr:class="dropdown-item" attr:style="text-decoration: none;">"Privacy Rules"</A>
                                <A href="/cookies" attr:class="dropdown-item" attr:style="text-decoration: none;">"Cookies"</A>
                                <A href="/contact" attr:class="dropdown-item" attr:style="text-decoration: none;">"Support"</A>
                                <div class="dropdown-divider"></div>
                                <div class="dropdown-item error" on:click=move |_| auth.logout()>
                                    <LogOut size={16} />
                                    {crate::text::TXT.action_sign_out}
                                </div>
                            </div>
                        </div>
                    </div>
            }.into_any(),
            None => view! {
                <div class="auth-cluster-guest" style="display: flex; gap: var(--s-3);">
                    <A href="/login" attr:class="btn btn-secondary">"Login"</A>
                    <A href="/register" attr:class="btn btn-primary">"Sign Up"</A>
                </div>
            }.into_any(),
        }}

        
    }
}

#[component]
fn AuthGuard(children: Children) -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    
    Effect::new(move |_| {
        if auth.session.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! { {children()} }
}

#[component]
fn Home() -> impl IntoView {
    let auth = use_auth();
    view! {
        <div class="fade-in" style="position: relative;">
            <div class="page-container" style="max-width: 1200px; margin: 0 auto; padding: 0 var(--s-8);">
                <div class="hero-section stagger-1">
                    <h1 class="hero-title text-gradient" style="font-size: 3.5rem; margin-bottom: var(--s-3); letter-spacing: -0.05em;">{crate::text::TXT.home_hero_title}</h1>
                    <p class="hero-subtitle muted" style="font-size: 1.0625rem; max-width: 600px;">{crate::text::TXT.home_hero_subtitle}</p>
                </div>
                
                <div class="home-showcase stagger-3">
                    <div class="showcase-frame-wrapper">
                        <div class="showcase-frame" style="height: 600px; display: flex; flex-direction: column; position: relative;">
                            <div class="slider-fill-home" style="flex: 1; min-height: 0;">
                                <ComparisonSlider 
                                    images=vec![
                                        ("./assets/hero_before_1.svg".to_string(), "./assets/hero_after_1.svg".to_string()),
                                        ("./assets/hero_before_2.svg".to_string(), "./assets/hero_after_2.svg".to_string()),
                                        ("./assets/hero_before_3.svg".to_string(), "./assets/hero_after_3.svg".to_string()),
                                        ("./assets/hero_before_4.svg".to_string(), "./assets/hero_after_4.svg".to_string()),
                                        ("./assets/hero_before_5.svg".to_string(), "./assets/hero_after_5.svg".to_string()),
                                    ]
                                />
                            </div>
                        </div>

                        <div class="showcase-actions-row">
                            <div class="action-meta">
                                <h3 class="action-title">"High-Fidelity AI Reconstruction"</h3>
                                <p class="action-desc">"Our Gemini-powered engine synthesizes high-frequency details where traditional upscalers fail."</p>
                            </div>
                            <div class="action-buttons">
                                <A href="/editor" attr:class="btn btn-primary cta-btn-hero">
                                    <Zap size={18} />
                                    <span>{crate::text::TXT.home_cta_start}</span>
                                </A>
                                {move || auth.user.get().is_some().then(|| view! {
                                    <A href="/history" attr:class="btn btn-secondary cta-btn-hero">
                                        <ImageIcon size={18} />
                                        <span>"VIEW GALLERY"</span>
                                    </A>
                                })}
                            </div>
                        </div>
                    </div>
                    


                    <div class="feature-grid stagger-5">
                        <div class="feature-item">
                            <div class="feature-icon"><Zap size={20} /></div>
                            <div class="feature-text">
                                <strong>"4K Synthesis"</strong>
                                <p>"Beyond scaling. Actual detail reconstruction."</p>
                            </div>
                        </div>
                        <div class="feature-item">
                            <div class="feature-icon"><Target size={20} /></div>
                            <div class="feature-text">
                                <strong>"Focal Locking"</strong>
                                <p>"Preserve your original depth of field."</p>
                            </div>
                        </div>
                        <div class="feature-item">
                            <div class="feature-icon"><RefreshCw size={20} /></div>
                            <div class="feature-text">
                                <strong>"Deterministic"</strong>
                                <p>"Perfectly reproducible seed-based results."</p>
                            </div>
                        </div>
                    </div>
                </div>

            
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
fn Credits() -> impl IntoView {
    view! {
        <crate::components::settings::Credits />
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="fade-in" style="text-align: center; padding: 10rem 0;">
            <h1 style="font-size: 5rem; font-weight: 800; opacity: 0.1;">"404"</h1>
            <h1 class="text-gradient" style="margin-top: -2.5rem; font-size: 2.5rem;">"Resource Not Found"</h1>
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

mod api;
mod auth;
mod components;
mod persistence;
mod text;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use crate::auth::{AuthProvider, use_auth};
use crate::api::ApiClient;
use leptos_router::hooks::use_location;
use crate::components::icons::{Zap, LogOut, Sun, Moon, UserIcon, Target, RefreshCw, ImageIcon};
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
}

pub fn provide_global_state() {
    let (quality, set_quality) = signal("2K".to_string());
    let (style, set_style) = signal("PHOTOGRAPHY".to_string());
    let (temperature, set_temperature) = signal(0.1);
    let (keep_depth_of_field, set_keep_depth_of_field) = signal(false);
    let (lighting, set_lighting) = signal("Original".to_string());
    let (thinking_level, set_thinking_level) = signal("MINIMAL".to_string());
    let (seed, set_seed) = signal(None::<u32>);
    let (temp_file, set_temp_file) = signal(None::<web_sys::File>);
    let (temp_classification, set_temp_classification) = signal(None::<String>);
    let (preview_base64, set_preview_base64) = signal(None::<String>);
    let (theme, set_theme) = signal("dark".to_string());

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
        let settings = persistence::UserSettings {
            quality: gs.quality.get(),
            style: gs.style.get(),
            temperature: gs.temperature.get(),
            keep_depth_of_field: gs.keep_depth_of_field.get(),
            lighting: gs.lighting.get(),
            thinking_level: gs.thinking_level.get(),
            seed: gs.seed.get(),
            theme: gs.theme.get(),
        };
        persistence::save_settings(&settings);
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
                    <div class="logo-icon"><Zap size={18} /></div>
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
                    <AuthNav />
                </nav>
            </header>

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
                        <div class="logo-icon"><crate::components::icons::Zap size={24} /></div>
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
            <style>
                "footer { border-top: 1px solid var(--glass-border); padding: var(--s-12) var(--s-12); margin-top: auto; background: hsl(var(--bg) / 0.8); backdrop-filter: blur(20px); }
                .footer-content { display: flex; justify-content: space-between; align-items: center; max-width: 1200px; margin: 0 auto; width: 100%; height: var(--s-12); padding: 0 var(--s-12); }
                
                .footer-left { flex: 1; display: flex; align-items: center; }
                .footer-logo { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text)); display: flex; align-items: center; gap: var(--s-2); letter-spacing: 0.2em; text-transform: uppercase; }
                
                .footer-center { flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--s-4); white-space: nowrap; }
                .footer-meta { font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; white-space: nowrap; }
                .footer-link { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-decoration: none; text-transform: uppercase; letter-spacing: 0.1em; transition: color 0.2s; white-space: nowrap; }
                .footer-link:hover { color: hsl(var(--accent)); }
                .divider { opacity: 0.1; color: hsl(var(--text-dim)); }

                .footer-right { flex: 1; display: flex; align-items: center; justify-content: flex-end; gap: var(--s-3); }
                .footer-status-tag { font-size: 0.625rem; font-weight: 850; letter-spacing: 0.1em; }
                .footer-status-tag.online { color: hsl(var(--success)); }
                .footer-status-tag.offline { color: hsl(var(--error)); }

                .status-indicator { width: 6px; height: 6px; border-radius: 50%; display: inline-block; }
                .status-indicator.online { background: hsl(var(--success)); }
                .status-indicator.offline { background: hsl(var(--error)); }
                
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
    let theme = use_global_state().theme;
    let set_theme = use_global_state().set_theme;
    
    view! {
        {move || match auth.user.get() {
            Some(user) => view! {
                    <div style="display: flex; align-items: center; gap: var(--s-6);">
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
                        
                        <div class="dropdown-container">
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
                                    {move || if theme.get() == "light" { "Dark Mode" } else { "Light Mode" }}
                                </div>
                                
                                <A href="/account" attr:class="dropdown-item" attr:style="text-decoration: none;">
                                    <UserIcon size={16} />
                                    "Profile Settings"
                                </A>
                                <div class="dropdown-item error" on:click=move |_| auth.logout()>
                                    <LogOut size={16} />
                                    {crate::text::TXT.action_sign_out}
                                </div>
                            </div>
                        </div>
                    </div>
            }.into_any(),
            None => view! {
                <div style="display: flex; gap: var(--s-3);">
                    <A href="/login" attr:class="btn btn-secondary">"Login"</A>
                    <A href="/register" attr:class="btn btn-primary">"Start Free"</A>
                </div>
            }.into_any(),
        }}

        <style>
            ".dropdown-container { position: relative; }
            .dropdown-menu { 
                position: absolute; top: calc(100% + 12px); right: 0; width: 220px; 
                background: hsl(var(--surface-raised)); 
                border: 1px solid var(--glass-border); border-radius: var(--radius-md);
                padding: 8px; box-shadow: var(--shadow-xl); z-index: 1000;
                transform-origin: top right; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
                opacity: 0; transform: translateY(10px) scale(0.95); pointer-events: none;
            }
            .dropdown-menu.show { opacity: 1; transform: translateY(0) scale(1); pointer-events: auto; }
            .dropdown-header { padding: 12px 16px; }
            .user-email { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-dim)); opacity: 0.8; }
            .dropdown-divider { height: 1px; background: var(--glass-border); margin: 8px 0; }
            .dropdown-item { 
                display: flex; align-items: center; gap: 12px; padding: 10px 16px;
                font-size: 0.8125rem; font-weight: 600; color: hsl(var(--text)); 
                border-radius: var(--radius-sm); cursor: pointer; transition: background 0.2s;
            }
            .dropdown-item:hover { background: rgba(255,255,255,0.05); }
            .dropdown-item.error { color: hsl(var(--error)); }
            .dropdown-item.error:hover { background: hsl(var(--error) / 0.1); }
            "
        </style>
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

            <style>
                ".home-showcase { 
                    max-width: 1100px;
                    margin: var(--s-10) auto 0;
                    display: flex;
                    flex-direction: column;
                    gap: var(--s-12);
                }

                .showcase-frame-wrapper {
                    padding: 1px;
                    background: linear-gradient(135deg, hsl(var(--accent) / 0.3) 0%, transparent 40%, hsl(var(--accent) / 0.1) 100%);
                    border-radius: calc(var(--radius-lg) + 1px);
                    box-shadow: 0 50px 120px -20px hsl(var(--accent) / 0.2);
                }
                
                .showcase-frame {
                    background: hsl(var(--surface));
                    border: 1px solid rgba(255,255,255,0.03);
                    border-radius: var(--radius-lg);
                    height: 580px;
                    overflow: hidden;
                    position: relative;
                }

                .showcase-actions-row {
                    padding: var(--s-10) var(--s-12);
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    gap: var(--s-12);
                    background: linear-gradient(135deg, rgba(255,255,255,0.02) 0%, transparent 100%);
                    border-top: 1px solid var(--glass-border);
                }

                .action-meta { flex: 1; text-align: left; }
                .action-title { font-size: 1.25rem; font-weight: 850; margin-bottom: var(--s-1); letter-spacing: -0.03em; color: hsl(var(--text)); }
                .action-desc { font-size: 0.9375rem; color: hsl(var(--text-dim)); line-height: 1.6; max-width: 500px; opacity: 0.7; }

                .action-buttons { display: flex; gap: var(--s-4); align-items: center; }
                
                .feature-grid {
                    display: grid;
                    grid-template-columns: repeat(3, 1fr);
                    gap: var(--s-12);
                    padding: var(--s-12);
                    background: rgba(255,255,255,0.02);
                    border-radius: var(--radius-lg);
                    border: 1px solid hsl(var(--accent) / 0.1);
                    margin-bottom: var(--s-20);
                }

                .feature-item { display: flex; gap: var(--s-6); align-items: flex-start; }
                .feature-icon { width: 48px; height: 48px; min-width: 48px; background: hsl(var(--accent) / 0.1); border-radius: 12px; display: flex; align-items: center; justify-content: center; color: hsl(var(--accent)); border: 1px solid hsl(var(--accent) / 0.1); }
                .feature-text strong { display: block; font-size: 0.875rem; color: white; margin-bottom: 4px; letter-spacing: 0.05em; text-transform: uppercase; }
                .feature-text p { font-size: 0.75rem; color: hsl(var(--text-dim)); line-height: 1.4; }

                .cta-btn-hero { 
                    height: 56px; padding: 0 var(--s-10); font-weight: 900; letter-spacing: 0.1em;
                    display: flex; align-items: center; gap: var(--s-3); white-space: nowrap;
                    min-width: fit-content;
                }
                
                @media (max-width: 1050px) {
                    .home-showcase { gap: var(--s-10); }
                    .showcase-actions { flex-direction: column; text-align: center; gap: var(--s-8); }
                    .action-meta { text-align: center; }
                    .action-desc { margin: 0 auto; }
                    .feature-grid { grid-template-columns: 1fr; gap: var(--s-8); }
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

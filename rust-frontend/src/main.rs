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
use crate::components::legal::{Terms, Contact};

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
                <MainLayout />
            </Router>
        </AuthProvider>
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    let auth = use_auth();
    
    view! {
        <div class="app-wrapper">
            <header class="glass stagger-1">
                <div class="logo">
                    <div class="logo-icon"><Zap size={18} /></div>
                    "UPSYL" 
                    <span>"STUDIO"</span>
                </div>
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
                    <span class="footer-meta">"© 2026 INFRA"</span>
                    <span class="divider">"|"</span>
                    <span class="footer-meta">"V1.0 ALPHA"</span>
                </div>

                <div class="footer-right">
                    <a href="/terms" class="footer-link">"Terms"</a>
                    <a href="/contact" class="footer-link">"Support"</a>
                </div>
            </div>
            <style>
                "footer { border-top: 1px solid var(--glass-border); padding: var(--s-6) var(--s-12); margin-top: auto; background: hsl(var(--bg) / 0.8); backdrop-filter: blur(20px); }
                .footer-content { display: flex; justify-content: space-between; align-items: center; max-width: 1200px; margin: 0 auto; width: 100%; height: var(--s-12); }
                
                .footer-left { flex: 1; display: flex; align-items: center; }
                .footer-logo { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text)); display: flex; align-items: center; gap: var(--s-2); letter-spacing: 0.2em; text-transform: uppercase; }
                
                .footer-center { flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--s-4); }
                .footer-meta { font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; }
                .divider { opacity: 0.1; color: hsl(var(--text-dim)); }

                .footer-right { flex: 1; display: flex; align-items: center; justify-content: flex-end; gap: var(--s-8); }
                .footer-link { font-size: 0.6875rem; font-weight: 800; color: hsl(var(--text-dim)); text-decoration: none; text-transform: uppercase; letter-spacing: 0.1em; transition: color 0.2s; }
                .footer-link:hover { color: hsl(var(--accent)); }
                
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
    
    let balance = LocalResource::new(
        move || {
            let session = auth.session.get();
            async move {
                if let Some(s) = session {
                    ApiClient::get_balance(Some(&s.access_token)).await
                } else {
                    std::future::pending::<Result<i32, String>>().await
                }
            }
        }
    );
    
    let (show_dropdown, set_show_dropdown) = signal(false);
    
    view! {
        {move || match auth.user.get() {
            Some(user) => Either::Left(view! {
                    <div style="display: flex; align-items: center; gap: var(--s-6);">
                        <Suspense>
                            {move || Suspend::new(async move {
                                let res = balance.get();
                                match res {
                                    Some(wrapper) => match &*wrapper {
                                        Ok(credits) => view! { 
                                            <div class="balance-pill">
                                                <Zap size={12} />
                                                <strong>{*credits}</strong>
                                                <span>"UNITS"</span>
                                            </div>
                                        }.into_any(),
                                        _ => ().into_any(),
                                    },
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
                <h1 class="text-gradient stagger-1">"Professional Super-Resolution"</h1>
                <p class="muted stagger-2" style="max-width: 680px; margin: 0 auto 4rem; font-size: 1.0625rem; font-weight: 600; line-height: 1.6; opacity: 0.8;">
                    "Bespoke neural restoration for photography and illustration. Restore frequency details, eliminate compression, and reach target resolutions with UPSYL precision."
                </p>
                
                <div class="hybrid-layout stagger-3">
                    <div class="hybrid-left">
                        <ComparisonSlider 
                            before_url="assets/hero_before.svg".to_string() 
                            after_url="assets/hero_after.svg".to_string() 
                            before_label="BEFORE (ORIGINAL)"
                            after_label="AFTER (UPSCALED)"
                        />
                    </div>
                    <div class="hybrid-right">
                        <div class="card hybrid-card">
                            <crate::components::upload_zone::UploadZone />
                        </div>
                    </div>
                </div>

            <style>
                ".hero-section { padding: var(--s-10) 0 var(--s-20); }
                .hybrid-layout { 
                    display: grid; 
                    grid-template-columns: 1.4fr 1fr; 
                    gap: var(--s-10); 
                    margin-top: var(--s-12); 
                    text-align: left;
                    align-items: stretch;
                    max-width: 1200px;
                    margin-left: auto;
                    margin-right: auto;
                }
                .hybrid-card { padding: var(--s-6); height: 100%; display: flex; flex-direction: column; }
                .hybrid-stats { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); margin-top: var(--s-4); }
                .h-stat { 
                    background: hsl(var(--surface-raised)); 
                    border: 1px solid hsl(var(--border)); 
                    padding: var(--s-4) var(--s-6); 
                    border-radius: var(--radius-md); 
                    transition: border-color 0.2s;
                }
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
    mount_to_body(App);
}

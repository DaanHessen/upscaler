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

#[component]
fn App() -> impl IntoView {
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
                            <Route path=path!("/history") view=History />
                            <Route path=path!("/settings") view=Credits />
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
                            on:click=move |_| set_show_dropdown.set(false)
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
    let auth = use_auth();

    view! {
        <div class="fade-in">
            {move || match auth.user.get() {
                Some(_) => Either::Left(view! {
                    <crate::components::upload_zone::Dashboard />
                }),
                None => Either::Right(view! {
                    <div class="hero-section">
                        <h1 class="text-gradient">"Professional Super-Resolution"</h1>
                        <p class="muted" style="max-width: 600px; margin: 0 auto 3rem; font-size: 1.125rem;">
                            "Bespoke neural upscaling for photography and illustration. 
                            Restore frequency details, eliminate compression, and reach target resolutions with surgical precision."
                        </p>
                        
                        <ComparisonSlider 
                            before_url="/assets/hero_before.png".to_string() 
                            after_url="/assets/hero_after.png".to_string() 
                            before_label="Original (Compressed)"
                            after_label="Upscaled (2K/4K)"
                        />

                        <div class="hero-actions" style="margin-top: 4rem;">
                            <a href="/login" class="btn btn-primary" style="padding: 1rem 3rem;">"Enter Studio"</a>
                        </div>
                    </div>
                    
                    <div class="stats-grid">
                        <div class="stat-card">
                            <span class="stat-label">"Pipeline"</span>
                            <span class="stat-value">"V7.1"</span>
                        </div>
                        <div class="stat-card">
                            <span class="stat-label">"Target"</span>
                            <span class="stat-value">"8K UHD"</span>
                        </div>
                        <div class="stat-card">
                            <span class="stat-label">"Infrastructure"</span>
                            <span class="stat-value">"STUDIO"</span>
                        </div>
                    </div>
                })
            }}
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

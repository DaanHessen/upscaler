use leptos::prelude::*;
use leptos::either::Either;
use leptos::task::spawn_local;
use crate::auth::use_auth;

#[component]
pub fn Login() -> impl IntoView {
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
                <h2>"Sign In"</h2>
                <p class="text-muted" style="margin-bottom: 2rem; font-size: 0.875rem;">"Access your upscaling workspace."</p>
                
                <button class="btn btn-secondary google-btn" style="width: 100%; margin-bottom: 1.5rem;">
                    <svg viewBox="0 0 24 24" width="18" height="18">
                        <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                        <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                        <path fill="#FBBC05" d="M5.84 14.1c-.22-.66-.35-1.36-.35-2.1s.13-1.44.35-2.1V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l3.66-2.84z"/>
                        <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.66l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                    </svg>
                    "Continue with Google"
                </button>

                <div class="divider"><span>"OR"</span></div>

                <form on:submit=on_submit>
                    <div class="input-group">
                        <label>"Email Address"</label>
                        <input 
                            type="email" 
                            placeholder="name@example.com"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=email
                            required
                        />
                    </div>
                    <div class="input-group" style="margin-top: 1.25rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                            <label style="margin-bottom: 0;">"Password"</label>
                            <a href="/forgot-password" class="text-link" style="font-size: 0.75rem;">"Forgot password?"</a>
                        </div>
                        <input 
                            type="password" 
                            placeholder="••••••••"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                            required
                        />
                    </div>
                    
                    {move || match error_msg.get() {
                        Some(msg) => Either::Left(view! {
                            <p class="error-text" style="margin-top: 1rem;">{msg}</p>
                        }.into_view()),
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

                <p class="auth-footer">
                    "Don't have an account? "
                    <a href="/register" class="text-link">"Create one"</a>
                </p>
            </div>

            <style>
                ".auth-container { padding: 8rem 0; }
                .divider { position: relative; text-align: center; margin: 1.5rem 0 2rem; }
                .divider::before { content: ''; position: absolute; top: 50%; left: 0; right: 0; height: 1px; background: var(--border-color); }
                .divider span { position: relative; background: var(--surface-color); padding: 0 1rem; color: var(--text-muted); font-size: 0.65rem; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; }
                
                .auth-footer { margin-top: 2.5rem; text-align: center; font-size: 0.8rem; color: var(--text-muted); font-weight: 500; }
                .text-link { color: var(--accent); text-decoration: none; font-weight: 700; }
                .text-link:hover { text-decoration: underline; }
                
                .input-group label { display: block; margin-bottom: 0.6rem; font-size: 0.65rem; font-weight: 800; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
                .input-group input { width: 100%; padding: 0.75rem 1rem; background: var(--bg-color) !important; color: var(--text-color); border: 1px solid var(--border-color); }
                .input-group input:focus { border-color: var(--accent); }
                
                .google-btn { background: white; color: #1f2328; gap: 0.75rem; border: 1px solid #d0d7de; padding: 0.75rem !important; font-size: 0.9rem; }
                .google-btn:hover { background: #f6f8fa; border-color: #d0d7de; }
                "
            </style>
        </div>
    }
}

#[component]
pub fn Register() -> impl IntoView {
    let auth = use_auth();
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error_msg.set(None);
        set_success_msg.set(None);
        
        let email_val = email.get();
        let pass_val = password.get();
        
        if pass_val != confirm_password.get() {
            set_error_msg.set(Some("Passwords do not match".to_string()));
            set_loading.set(false);
            return;
        }
        
        spawn_local(async move {
            match auth.signup(&email_val, &pass_val).await {
                Ok(_) => {
                    set_success_msg.set(Some("Registration successful. Please check your email for confirmation.".to_string()));
                    set_loading.set(false);
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
                <h2>"Create Account"</h2>
                <p class="text-muted" style="margin-bottom: 2rem; font-size: 0.875rem;">"Join our professional upscaling studio."</p>

                <button class="btn btn-secondary google-btn" style="width: 100%; margin-bottom: 1.5rem;">
                    <svg viewBox="0 0 24 24" width="18" height="18">
                        <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                        <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                        <path fill="#FBBC05" d="M5.84 14.1c-.22-.66-.35-1.36-.35-2.1s.13-1.44.35-2.1V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l3.66-2.84z"/>
                        <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.66l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                    </svg>
                    "Sign up with Google"
                </button>

                <div class="divider"><span>"OR"</span></div>

                <form on:submit=on_submit>
                    <div class="input-group">
                        <label>"Email Address"</label>
                        <input 
                            type="email" 
                            placeholder="name@example.com"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=email
                            required
                        />
                    </div>
                    <div class="input-group" style="margin-top: 1.25rem;">
                        <label>"Password"</label>
                        <input 
                            type="password" 
                            placeholder="Create a password"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                            required
                        />
                    </div>
                    <div class="input-group" style="margin-top: 1.25rem;">
                        <label>"Confirm Password"</label>
                        <input 
                            type="password" 
                            placeholder="Repeat password"
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            prop:value=confirm_password
                            required
                        />
                    </div>
                    
                    {move || match error_msg.get() {
                        Some(msg) => Either::Left(view! {
                            <p class="error-text" style="margin-top: 1rem;">{msg}</p>
                        }.into_view()),
                        None => Either::Right(())
                    }}

                    {move || match success_msg.get() {
                        Some(msg) => Either::Left(view! {
                            <div class="success-panel" style="margin-top: 1.5rem;">
                                <p>{msg}</p>
                                <a href="/login" class="btn btn-primary" style="margin-top: 1rem; width: 100%;">"Return to Login"</a>
                            </div>
                        }.into_view()),
                        None => Either::Right(view! {
                            <button 
                                type="submit" 
                                class="btn btn-primary" 
                                style="margin-top: 2rem; width: 100%;"
                                disabled=loading
                            >
                                {move || if loading.get() { "Creating Account..." } else { "Create Account" }}
                            </button>
                        })
                    }}
                </form>

                <p class="auth-footer">
                    "Already have an account? "
                    <a href="/login" class="text-link">"Sign in"</a>
                </p>
            </div>
            
            <style>
                ".success-panel { padding: 1.5rem; background: rgba(63, 185, 80, 0.05); border: 1px solid rgba(63, 185, 80, 0.2); border-radius: 8px; text-align: center; }
                .success-panel p { font-size: 0.85rem; color: var(--success); font-weight: 600; line-height: 1.4; }
                "
            </style>
        </div>
    }
}

#[component]
pub fn ForgotPassword() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (submitted, set_submitted) = signal(false);

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        // Placeholder for password reset logic
        set_submitted.set(true);
    };

    view! {
        <div class="fade-in auth-container">
            <div class="card auth-card">
                <h2>"Reset Password"</h2>
                
                {move || if submitted.get() {
                    Either::Left(view! {
                        <div class="success-panel">
                            <p>"If an account exists for " <strong>{email.get()}</strong> ", you will receive a password reset link shortly."</p>
                            <a href="/login" class="btn btn-primary" style="margin-top: 2rem; width: 100%;">"Return to Login"</a>
                        </div>
                    })
                } else {
                    Either::Right(view! {
                        <p class="text-muted" style="margin-bottom: 2rem; font-size: 0.875rem;">"Enter your email to receive a reset link."</p>
                        <form on:submit=on_submit>
                            <div class="input-group">
                                <label>"Email Address"</label>
                                <input 
                                    type="email" 
                                    placeholder="name@example.com"
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    prop:value=email
                                    required
                                />
                            </div>
                            <button type="submit" class="btn btn-primary" style="margin-top: 2rem; width: 100%;">"Send Reset Link"</button>
                            <a href="/login" class="btn btn-secondary" style="margin-top: 1rem; width: 100%;">"Back to Login"</a>
                        </form>
                    })
                }}
            </div>
        </div>
    }
}

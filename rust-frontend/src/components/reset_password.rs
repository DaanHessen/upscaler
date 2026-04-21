use leptos::prelude::*;
use crate::auth::use_auth;
use crate::components::icons::{Lock, ShieldCheck};

#[component]
pub fn ResetPassword() -> impl IntoView {
    let auth = use_auth();
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);
    let (success, set_success) = signal(false);

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        
        if password.get() != confirm_password.get() {
            set_error.set(Some("Passwords do not match".to_string()));
            return;
        }

        if password.get().len() < 6 {
            set_error.set(Some("Password must be at least 6 characters".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let ctx = auth;
        let pw = password.get();

        leptos::task::spawn_local(async move {
            match ctx.update_password(&pw).await {
                Ok(_) => {
                    set_success.set(true);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="fade-in auth-container">
            <div class="card auth-card">
                <h2>"New Password"</h2>
                <p class="text-muted" style="margin-bottom: 2rem; font-size: 0.875rem;">"Establish your new security credentials for UPSYL STUDIO."</p>

                {move || if success.get() {
                    view! {
                        <div class="success-panel" style="text-align: center;">
                            <div style="color: hsl(var(--success)); margin-bottom: var(--s-6);">
                                <ShieldCheck size={48} />
                            </div>
                            <h3>"Identity Secured"</h3>
                            <p>"Your password has been updated successfully. You can now sign in with your new credentials."</p>
                            <a href="/login" class="btn btn-primary" style="margin-top: 2rem; width: 100%; text-decoration: none; display: block; text-align: center;">"RETURN TO LOGIN"</a>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form on:submit=on_submit>
                            <div class="input-group">
                                <label>"NEW PASSWORD"</label>
                                <div style="position: relative;">
                                    <input 
                                        type="password" 
                                        placeholder="••••••••"
                                        on:input=move |ev| set_password.set(event_target_value(&ev))
                                        prop:value=password
                                        required
                                        style="padding-left: 3rem;"
                                    />
                                    <div style="position: absolute; left: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.3;">
                                        <Lock size={16} />
                                    </div>
                                </div>
                            </div>

                            <div class="input-group" style="margin-top: var(--s-4);">
                                <label>"CONFIRM PASSWORD"</label>
                                <div style="position: relative;">
                                    <input 
                                        type="password" 
                                        placeholder="••••••••"
                                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                        prop:value=confirm_password
                                        required
                                        style="padding-left: 3rem;"
                                    />
                                    <div style="position: absolute; left: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.3;">
                                        <Lock size={16} />
                                    </div>
                                </div>
                            </div>

                            {move || error.get().map(|e| view! {
                                <div class="error-msg" style="margin-top: var(--s-4);">
                                    {e}
                                </div>
                            })}

                            <button 
                                type="submit" 
                                class="btn btn-primary" 
                                disabled=loading.get()
                                style="margin-top: 2rem; width: 100%;"
                            >
                                {move || if loading.get() { "SECURING..." } else { "UPDATE PASSWORD" }}
                            </button>
                        </form>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

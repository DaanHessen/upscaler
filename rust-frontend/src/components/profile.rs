use leptos::prelude::*;
use crate::auth::use_auth;
use crate::components::icons::{UserIcon, Lock, ShieldCheck};

#[component]
pub fn AccountSettings() -> impl IntoView {
    let auth = use_auth();
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);
    let (success, set_success) = signal(false);

    let on_update_password = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        if new_password.get() != confirm_password.get() {
            set_error.set(Some("Passwords do not match".to_string()));
            return;
        }
        if new_password.get().len() < 6 {
            set_error.set(Some("Must be at least 6 characters".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);
        let ctx = auth;
        let pw = new_password.get();

        leptos::task::spawn_local(async move {
            match ctx.update_password(&pw).await {
                Ok(_) => set_success.set(true),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="fade-in container-narrow" style="padding: var(--s-12) 0;">
            <div class="history-header">
                <div class="header-main">
                    <h1 class="text-gradient">"Account Settings"</h1>
                    <p class="muted">"Manage your studio identity and security."</p>
                </div>
            </div>

            <div class="settings-grid" style="display: grid; gap: var(--s-8); margin-top: var(--s-8);">
                /* Identity Section */
                <div class="card" style="padding: var(--s-8);">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-6);">
                        <div style="background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); padding: var(--s-3); border-radius: var(--radius-md);">
                            <UserIcon size={20} />
                        </div>
                        <h3 style="margin: 0; font-size: 1.125rem;">"Identity"</h3>
                    </div>
                    
                    <div style="display: grid; gap: var(--s-4);">
                        <div class="data-row">
                            <span class="data-label">"USER ID"</span>
                            <span class="data-value">{move || auth.user.get().map(|u| u.id).unwrap_or_default()}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">"EMAIL ADDRESS"</span>
                            <span class="data-value">{move || auth.user.get().and_then(|u| u.email).unwrap_or_default()}</span>
                        </div>
                    </div>
                </div>

                /* Security Section */
                <div class="card" style="padding: var(--s-8);">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-6);">
                        <div style="background: hsl(var(--warning) / 0.1); color: hsl(var(--warning)); padding: var(--s-3); border-radius: var(--radius-md);">
                            <Lock size={20} />
                        </div>
                        <h3 style="margin: 0; font-size: 1.125rem;">"Security"</h3>
                    </div>

                    {move || if success.get() {
                        view! {
                            <div class="success-panel" style="padding: var(--s-4); background: hsl(var(--success) / 0.05); border-radius: var(--radius-md); border: 1px solid hsl(var(--success) / 0.1); display: flex; align-items: center; gap: var(--s-4);">
                                <ShieldCheck size={20} custom_style="color: hsl(var(--success));".to_string() />
                                <span style="font-size: 0.875rem; font-weight: 600;">"Password updated successfully."</span>
                                <button class="btn btn-secondary btn-sm" style="margin-left: auto;" on:click=move |_| set_success.set(false)>"DISMISS"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <form on:submit=on_update_password>
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4);">
                                    <div class="input-group">
                                        <label>"NEW PASSWORD"</label>
                                        <input 
                                            type="password" 
                                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                            prop:value=new_password
                                            required
                                        />
                                    </div>
                                    <div class="input-group">
                                        <label>"CONFIRM PASSWORD"</label>
                                        <input 
                                            type="password" 
                                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                            prop:value=confirm_password
                                            required
                                        />
                                    </div>
                                </div>
                                {move || error.get().map(|e| view! { <p style="color: hsl(var(--error)); font-size: 0.75rem; margin-top: var(--s-2); font-weight: 700;">{e}</p> })}
                                <button type="submit" class="btn btn-secondary btn-sm" style="margin-top: var(--s-6);" disabled=loading.get()>
                                    {move || if loading.get() { "UPDATING..." } else { "CHANGE PASSWORD" }}
                                </button>
                            </form>
                        }.into_any()
                    }}
                </div>

                /* Advanced Section */
                <div class="card" style="padding: var(--s-8); background: hsl(var(--error) / 0.02); border-color: hsl(var(--error) / 0.1);">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-4);">
                        <h3 style="margin: 0; font-size: 1rem; color: hsl(var(--error));">"Danger Zone"</h3>
                    </div>
                    <p class="muted" style="font-size: 0.8125rem; margin-bottom: var(--s-6);">"Irreversibly delete your Upsyl Studio account and all stored history. This action cannot be undone."</p>
                    <button class="btn btn-secondary btn-sm" style="color: hsl(var(--error)); border-color: hsl(var(--error) / 0.2);" on:click=move |_| {
                        let win = web_sys::window().unwrap();
                        win.alert_with_message("Account deletion is a manual process. Please contact support@upsyl.com to initiate a deletion request.").unwrap();
                    }>"DELETE ACCOUNT"</button>
                </div>
            </div>

            <style>
                ".container-narrow { max-width: 800px; margin: 0 auto; width: 100%; }
                .data-row { display: flex; justify-content: space-between; padding: var(--s-3) 0; border-bottom: 1px solid var(--glass-border); }
                .data-row:last-child { border-bottom: none; }
                .data-label { font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .data-value { font-size: 0.8125rem; font-weight: 600; color: hsl(var(--text)); font-family: var(--font-mono); }
                "
            </style>
        </div>
    }
}

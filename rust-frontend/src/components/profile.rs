use leptos::prelude::*;
use crate::auth::use_auth;
use crate::components::icons::UserIcon;

#[component]
pub fn AccountSettings() -> impl IntoView {
    let auth = use_auth();
    let (new_password, _set_new_password) = signal(String::new());
    let (confirm_password, _set_confirm_password) = signal(String::new());
    let (_error, _set_error) = signal(Option::<String>::None);
    let (_loading, _set_loading) = signal(false);
    let (_success, _set_success) = signal(false);

    let _on_update_password = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        if new_password.get() != confirm_password.get() {
            _set_error.set(Some("Passwords do not match".to_string()));
            return;
        }
        if new_password.get().len() < 6 {
            _set_error.set(Some("Must be at least 6 characters".to_string()));
            return;
        }

        _set_loading.set(true);
        _set_error.set(None);
        let ctx = auth;
        let pw = new_password.get();

        leptos::task::spawn_local(async move {
            match ctx.update_password(&pw).await {
                Ok(_) => _set_success.set(true),
                Err(e) => _set_error.set(Some(e)),
            }
            _set_loading.set(false);
        });
    };

    view! {
        <div class="fade-in settings-container">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"Account Settings"</h1>
                    <p class="muted stagger-2">"Manage your studio identity and security."</p>
                </div>
            </div>

            <div class="settings-grid" style="display: grid; gap: var(--s-8); margin-top: var(--s-8);">
                /* Identity Section */
                <div class="card shadow-lg" style="padding: var(--s-10); border: 1px solid var(--border); background: hsl(var(--surface));">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-6);">
                        <div style="color: hsl(var(--text)); padding: 0; display: flex; align-items: center; justify-content: center;">
                            <UserIcon size={24} />
                        </div>
                        <h3 style="margin: 0; font-size: 1.25rem; font-weight: 800; letter-spacing: -0.02em; color: hsl(var(--text));">"Identity"</h3>
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
                        <div class="data-row">
                            <span class="data-label">"ACCOUNT STATUS"</span>
                            <span class="data-value" style="color: hsl(var(--success));">"ACTIVE / VERIFIED"</span>
                        </div>
                    </div>
                </div>

                /* Statistics Section */
                <div class="card shadow-lg" style="padding: var(--s-10); margin-bottom: var(--s-6);">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-8);">
                        <h3 style="margin: 0; font-size: 1.25rem; font-weight: 800; letter-spacing: -0.02em; color: hsl(var(--text));">"Usage Statistics"</h3>
                    </div>
                    
                    <div class="stats-grid-mini" style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-8);">
                        <div class="stat-box-mini">
                            <span class="stat-label-mini" style="font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; display: block; margin-bottom: 4px;">"Processed Images"</span>
                            <span class="stat-value-mini" style="font-size: 1.5rem; font-weight: 800; font-family: var(--font-heading);">{move || auth.history.get().map(|h| h.len()).unwrap_or(0)} <span style="font-size: 0.75rem; opacity: 0.4;">"IMAGES"</span></span>
                        </div>
                        <div class="stat-box-mini">
                            <span class="stat-label-mini" style="font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; display: block; margin-bottom: 4px;">"Total Credits Used"</span>
                            <span class="stat-value-mini" style="font-size: 1.5rem; font-weight: 800; font-family: var(--font-heading);">{move || auth.history.get().map(|h| h.iter().map(|i| i.credits_charged).sum::<i32>()).unwrap_or(0)} <span style="font-size: 0.75rem; opacity: 0.4;">"CREDITS"</span></span>
                        </div>
                        <div class="stat-box-mini">
                            <span class="stat-label-mini" style="font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; display: block; margin-bottom: 4px;">"Average Speed"</span>
                            <span class="stat-value-mini" style="font-size: 1.5rem; font-weight: 800; font-family: var(--font-heading);">
                                {move || {
                                    let h = auth.history.get().unwrap_or_default();
                                    if h.is_empty() { "0.0".to_string() } else {
                                        let total_ms: i32 = h.iter().map(|i| i.latency_ms).sum();
                                        format!("{:.1}", (total_ms as f32 / h.len() as f32) / 1000.0)
                                    }
                                }}
                                <span style="font-size: 0.75rem; opacity: 0.4;">"SEC AVG"</span>
                            </span>
                        </div>
                        <div class="stat-box-mini">
                            <span class="stat-label-mini" style="font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; display: block; margin-bottom: 4px;">"System Status"</span>
                            <span class="stat-value-mini" style="font-size: 0.8125rem; font-weight: 800; color: hsl(var(--accent));">"SYNCED"</span>
                        </div>
                    </div>
                </div>



                /* Advanced Section */
                <div class="card shadow-lg" style="padding: var(--s-10); background: hsl(var(--error) / 0.05); border: 1px solid hsl(var(--error) / 0.2);">
                    <div style="display: flex; align-items: center; gap: var(--s-4); margin-bottom: var(--s-4);">
                        <h3 style="margin: 0; font-size: 1.25rem; font-weight: 800; letter-spacing: -0.02em; color: hsl(var(--error));">"Danger Zone"</h3>
                    </div>
                    <p class="muted" style="font-size: 0.875rem; margin-bottom: var(--s-6); max-width: 600px;">"Irreversibly delete your Upsyl Studio account and all stored history. This action cannot be undone."</p>
                    <button class="btn btn-secondary" style="color: hsl(var(--error)); border-color: hsl(var(--error) / 0.3);" on:click=move |_| {
                        let win = web_sys::window().unwrap();
                        win.alert_with_message("Account deletion is a manual process. Please contact support@upsyl.com to initiate a deletion request.").unwrap();
                    }>"DELETE ACCOUNT"</button>
                </div>
            </div>

            <style>
                ".settings-container { max-width: 1200px; margin: 0 auto; width: 100%; padding: 0 var(--s-8) var(--s-20) var(--s-8); }
                .input-group label { font-size: 0.75rem; font-weight: 850; text-transform: uppercase; letter-spacing: 0.05em; color: hsl(var(--text-dim)); margin-bottom: var(--s-2); display: block; }
                .input-group input { padding: var(--s-3) var(--s-4); border: 1px solid var(--border); color: hsl(var(--text)); transition: border-color 0.2s, box-shadow 0.2s; }
                .input-group input:focus { outline: none; border-color: hsl(var(--accent)); box-shadow: 0 0 0 2px hsl(var(--accent) / 0.1); }
                .data-row { display: flex; justify-content: space-between; padding: var(--s-4) 0; border-bottom: 1px dashed var(--border); }
                .data-row:last-child { border-bottom: none; }
                .data-label { font-size: 0.6875rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .data-value { font-size: 0.875rem; font-weight: 700; color: hsl(var(--text)); font-family: var(--font-mono); }
                "
            </style>
        </div>
    }
}

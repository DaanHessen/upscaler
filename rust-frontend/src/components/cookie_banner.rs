use leptos::prelude::*;
use leptos_router::components::A;
use crate::components::icons::Info;

#[component]
pub fn CookieBanner() -> impl IntoView {
    let (is_visible, set_is_visible) = signal(false);
    let (show_settings, set_show_settings) = signal(false);
    let (analytics_enabled, set_analytics_enabled) = signal(false);
    let (marketing_enabled, set_marketing_enabled) = signal(false);

    // Initialize state on client side
    Effect::new(move |_| {
        let win = web_sys::window().unwrap();
        let store = win.local_storage().unwrap().unwrap();
        
        if let Ok(Some(val)) = store.get_item("cookie_consent") {
            if val == "accepted" || val == "custom" {
                set_is_visible.set(false);
                if let Ok(Some(settings_val)) = store.get_item("cookie_settings") {
                    if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&settings_val) {
                        if let Some(a) = settings.get("analytics").and_then(|v| v.as_bool()) {
                            set_analytics_enabled.set(a);
                        }
                        if let Some(m) = settings.get("marketing").and_then(|v| v.as_bool()) {
                            set_marketing_enabled.set(m);
                        }
                    }
                }
                return;
            }
        }
        set_is_visible.set(true);
    });

    let accept_all = move |_| {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(store)) = win.local_storage() {
                let _ = store.set_item("cookie_consent", "accepted");
                let settings = serde_json::json!({
                    "essential": true,
                    "analytics": true,
                    "marketing": true
                });
                let _ = store.set_item("cookie_settings", &settings.to_string());
            }
        }
        set_is_visible.set(false);
        set_show_settings.set(false);
    };

    let save_settings = move |_| {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(store)) = win.local_storage() {
                let _ = store.set_item("cookie_consent", "custom");
                let settings = serde_json::json!({
                    "essential": true,
                    "analytics": analytics_enabled.get(),
                    "marketing": marketing_enabled.get()
                });
                let _ = store.set_item("cookie_settings", &settings.to_string());
            }
        }
        set_is_visible.set(false);
        set_show_settings.set(false);
    };

    view! {
        {move || is_visible.get().then(|| view! {
            <div class="cookie-banner-v3 fade-in" style="
                position: fixed;
                bottom: 32px;
                left: 0;
                right: 0;
                margin: 0 auto;
                background: var(--glass);
                backdrop-filter: blur(30px) saturate(180%);
                border: 1px solid var(--glass-border);
                border-radius: 100px;
                padding: 10px 10px 10px 20px;
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 16px;
                box-shadow: 0 20px 40px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.05);
                z-index: 9999;
                width: max-content;
                max-width: 95vw;
                flex-wrap: wrap;
            ">
                <div style="color: hsl(var(--accent)); display: flex; align-items: center;">
                    <Info size={16} />
                </div>
                
                <div style="display: flex; align-items: center; gap: 16px; flex-wrap: wrap; justify-content: center;">
                    <span style="font-size: 0.75rem; color: hsl(var(--text-dim)); font-weight: 600;">
                        "We use cookies to improve your experience."
                    </span>
                    <div style="display: flex; gap: 12px; align-items: center;">
                        <A href="/privacy" attr:style="font-size: 0.6875rem; font-weight: 800; color: hsl(var(--text)); opacity: 0.6; text-decoration: none; text-transform: uppercase; letter-spacing: 0.05em;">"Privacy"</A>
                        <span style="width: 4px; height: 4px; border-radius: 50%; background: hsl(var(--text-dim) / 0.3);"></span>
                        <A href="/cookies" attr:style="font-size: 0.6875rem; font-weight: 800; color: hsl(var(--text)); opacity: 0.6; text-decoration: none; text-transform: uppercase; letter-spacing: 0.05em;">"Cookies"</A>
                    </div>
                </div>

                <div class="cb-divider" style="width: 1px; height: 16px; background: var(--glass-border); margin: 0 4px;"></div>
                
                <div style="display: flex; gap: 8px;">
                    <button style="background: transparent; border: 1px solid var(--glass-border); color: hsl(var(--text)); padding: 8px 16px; border-radius: 100px; font-size: 0.8125rem; font-weight: 600; cursor: pointer; white-space: nowrap;" on:click=move |_| set_show_settings.set(true)>
                        "Settings"
                    </button>
                    <button class="viewer-action-btn" style="white-space: nowrap; padding: 8px 20px;" on:click=accept_all>
                        "Accept All"
                    </button>
                </div>
            </div>
        })}

        {move || show_settings.get().then(|| view! {
            <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.8); z-index: 10000; display: flex; align-items: center; justify-content: center; backdrop-filter: blur(12px);">
                <div class="fade-in" style="background: hsl(var(--surface-bright)); border: 1px solid var(--border); border-radius: 20px; width: 100%; max-width: 540px; max-height: 90vh; overflow-y: auto; padding: 32px; display: flex; flex-direction: column; gap: 28px; box-shadow: 0 30px 60px rgba(0,0,0,0.8), 0 0 0 1px rgba(255,255,255,0.05); position: relative;">
                    
                    <button 
                        on:click=move |_| set_show_settings.set(false)
                        style="position: absolute; top: 24px; right: 24px; background: transparent; border: none; color: hsl(var(--text-dim)); cursor: pointer; padding: 4px; border-radius: 50%; display: flex; align-items: center; justify-content: center; transition: all 0.2s;"
                        onmouseover="this.style.color='hsl(var(--text))'; this.style.background='rgba(255,255,255,0.1)';"
                        onmouseout="this.style.color='hsl(var(--text-dim))'; this.style.background='transparent';"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>

                    <div style="display: flex; flex-direction: column; gap: 10px; padding-right: 24px;">
                        <h2 style="margin: 0; font-size: 1.5rem; font-weight: 800; letter-spacing: -0.02em;">"Cookie Preferences"</h2>
                        <p style="margin: 0; color: hsl(var(--text-dim)); font-size: 0.9375rem; line-height: 1.5;">
                            "We use cookies to enhance your experience. Choose which cookies you want to allow. Essential cookies are necessary for the website to function properly."
                        </p>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 16px;">
                        
                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 20px; background: var(--glass); border-radius: 16px; border: 1px solid var(--glass-border);">
                            <div style="display: flex; flex-direction: column; gap: 6px;">
                                <h3 style="margin: 0; font-size: 1rem; font-weight: 600;">"Strictly Necessary"</h3>
                                <p style="margin: 0; color: hsl(var(--text-dim)); font-size: 0.8125rem; line-height: 1.4;">"Required for basic functionality such as user authentication and keeping you logged in. Cannot be disabled."</p>
                            </div>
                            <div style="opacity: 0.6; pointer-events: none; flex-shrink: 0;">
                                <button type="button" role="switch" aria-checked="true" disabled=true style="position: relative; width: 44px; height: 24px; border-radius: 9999px; border: none; background: hsl(var(--accent)); opacity: 0.5; padding: 0; display: inline-flex; align-items: center;">
                                    <span style="position: absolute; left: 2px; width: 20px; height: 20px; background: white; border-radius: 50%; transform: translateX(20px);"></span>
                                </button>
                            </div>
                        </div>

                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 20px; background: var(--glass); border-radius: 16px; border: 1px solid var(--glass-border);">
                            <div style="display: flex; flex-direction: column; gap: 6px;">
                                <h3 style="margin: 0; font-size: 1rem; font-weight: 600;">"Analytics"</h3>
                                <p style="margin: 0; color: hsl(var(--text-dim)); font-size: 0.8125rem; line-height: 1.4;">"Helps us understand how visitors interact with the site by collecting and reporting information anonymously."</p>
                            </div>
                            <div style="flex-shrink: 0;">
                                <button type="button" role="switch" aria-checked=analytics_enabled.get().to_string() on:click=move |_| set_analytics_enabled.update(|v| *v = !*v) style=move || format!("position: relative; width: 44px; height: 24px; border-radius: 9999px; border: none; cursor: pointer; background: {}; padding: 0; display: inline-flex; align-items: center; transition: background 0.2s ease;", if analytics_enabled.get() { "hsl(var(--accent))" } else { "rgba(255,255,255,0.15)" })>
                                    <span style=move || format!("position: absolute; left: 2px; width: 20px; height: 20px; background: white; border-radius: 50%; transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1); transform: translateX({}); box-shadow: 0 2px 4px rgba(0,0,0,0.2);", if analytics_enabled.get() { "20px" } else { "0px" })></span>
                                </button>
                            </div>
                        </div>

                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 20px; background: var(--glass); border-radius: 16px; border: 1px solid var(--glass-border);">
                            <div style="display: flex; flex-direction: column; gap: 6px;">
                                <h3 style="margin: 0; font-size: 1rem; font-weight: 600;">"Marketing"</h3>
                                <p style="margin: 0; color: hsl(var(--text-dim)); font-size: 0.8125rem; line-height: 1.4;">"Used to track visitors across websites to display relevant advertisements and track their performance."</p>
                            </div>
                            <div style="flex-shrink: 0;">
                                <button type="button" role="switch" aria-checked=marketing_enabled.get().to_string() on:click=move |_| set_marketing_enabled.update(|v| *v = !*v) style=move || format!("position: relative; width: 44px; height: 24px; border-radius: 9999px; border: none; cursor: pointer; background: {}; padding: 0; display: inline-flex; align-items: center; transition: background 0.2s ease;", if marketing_enabled.get() { "hsl(var(--accent))" } else { "rgba(255,255,255,0.15)" })>
                                    <span style=move || format!("position: absolute; left: 2px; width: 20px; height: 20px; background: white; border-radius: 50%; transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1); transform: translateX({}); box-shadow: 0 2px 4px rgba(0,0,0,0.2);", if marketing_enabled.get() { "20px" } else { "0px" })></span>
                                </button>
                            </div>
                        </div>

                    </div>

                    <div style="display: flex; justify-content: flex-end; gap: 12px; margin-top: 12px; padding-top: 24px; border-top: 1px solid var(--glass-border);">
                        <button style="background: transparent; border: 1px solid var(--glass-border); color: hsl(var(--text)); padding: 10px 24px; border-radius: 100px; font-size: 0.875rem; font-weight: 600; cursor: pointer; transition: background 0.2s;" onmouseover="this.style.background='rgba(255,255,255,0.05)'" onmouseout="this.style.background='transparent'" on:click=move |_| set_show_settings.set(false)>
                            "Cancel"
                        </button>
                        <button style="background: hsl(var(--accent)); color: white; border: none; padding: 10px 24px; border-radius: 100px; font-size: 0.875rem; font-weight: 600; cursor: pointer; transition: opacity 0.2s;" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'" on:click=save_settings>
                            "Save Preferences"
                        </button>
                    </div>

                </div>
            </div>
        })}
    }
}

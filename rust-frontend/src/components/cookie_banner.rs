use leptos::prelude::*;
use leptos_router::components::A;
use crate::components::icons::Info;

#[component]
pub fn CookieBanner() -> impl IntoView {
    let (is_visible, set_is_visible) = signal(false);

    // Initialize state on client side
    Effect::new(move |_| {
        let win = web_sys::window().unwrap();
        let store = win.local_storage().unwrap().unwrap();
        
        if let Ok(Some(val)) = store.get_item("cookie_consent") {
            if val == "accepted" {
                set_is_visible.set(false);
                return;
            }
        }
        set_is_visible.set(true);
    });

    let accept = move |_| {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(store)) = win.local_storage() {
                let _ = store.set_item("cookie_consent", "accepted");
            }
        }
        set_is_visible.set(false);
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
                
                <button class="viewer-action-btn" style="white-space: nowrap; padding: 8px 20px;" on:click=accept>
                    "Accept"
                </button>
            </div>
        })}
    }
}

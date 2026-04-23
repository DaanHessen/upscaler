use leptos::prelude::*;
use crate::components::icons::{Zap, CreditCard};
use crate::auth::use_auth;
use crate::api::ApiClient;

#[component]
pub fn Credits() -> impl IntoView {
    let auth = use_auth();
    
    // Trigger throttled telemetry sync on mount (handles SPA navigation)
    Effect::new(move |_| {
        auth.sync_telemetry(false);
    });

    let (selected_pack, set_selected_pack) = signal(10); // Default to 10 euro pack
    let (loading, set_loading) = signal(false);

    let on_buy = move |_| {
        let auth = auth.clone();
        let tier = selected_pack.get().to_string();
        set_loading.set(true);

        leptos::task::spawn_local(async move {
            let token = auth.session.get().map(|s| s.access_token);
            let window = web_sys::window().unwrap();
            let location = window.location();
            let origin = location.origin().unwrap();
            let success_url = format!("{}/settings?success=true", origin);
            let cancel_url = format!("{}/settings?cancel=true", origin);

            match ApiClient::create_checkout_session(token.as_deref(), &tier, &success_url, &cancel_url).await {
                Ok(url) => {
                    let _ = location.set_href(&url);
                }
                Err(e) => {
                    leptos::logging::error!("Checkout failed: {}", e);
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="credits-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"Credits & Usage"</h1>
                    <p class="muted stagger-2">"Manage your credits and view your activity history."</p>
                </div>
            </div>

            <div class="card unified-billing-card shadow-lg" style="margin-top: var(--s-8);">
                <div class="unified-billing-grid" style="display: grid; grid-template-columns: 1fr 1fr;">
                    /* Balance Section */
                    <div class="billing-section balance-section" style="padding: var(--s-10) var(--s-12); border-right: 1px solid hsl(var(--border) / 0.5); display: flex; flex-direction: column;">
                        <div class="card-tag">
                            <Zap size={10} />
                            <span>"VAULT BALANCE"</span>
                        </div>
                        <div class="balance-display" style="margin-top: var(--s-12); display: flex; flex-direction: column; align-items: flex-start;">
                            <span class="credit-count" style="font-size: 5.5rem; line-height: 0.9; font-family: var(--font-heading); font-weight: 800; letter-spacing: -0.04em;">
                                {move || auth.credits.get().map(|c| c.to_string()).unwrap_or_else(|| "---".to_string())}
                            </span>
                            <span class="credit-symbol" style="margin-top: var(--s-2); font-size: 0.875rem; font-weight: 600; color: hsl(var(--text-dim));">"Credits Available"</span>
                        </div>
                        
                        <div class="meta-stats" style="margin-top: auto; padding-top: var(--s-8); border-top: 1px solid hsl(var(--border-muted));">
                            <div class="stat-box">
                                <span class="stat-label">"Last Top Up"</span>
                                <span class="stat-value" style="font-size: 0.9375rem;">"N/A"</span>
                            </div>
                            <div class="stat-box">
                                <span class="stat-label">"Status"</span>
                                <span class="stat-value highlight" style="font-size: 0.9375rem;">"SYNCED"</span>
                            </div>
                        </div>
                    </div>

                    /* Replenish Section */
                    <div class="billing-section replenish-section" style="padding: var(--s-10) var(--s-12); display: flex; flex-direction: column;">
                        <div class="card-tag" style="margin-bottom: var(--s-8);">
                            <CreditCard size={10} />
                            <span>"BUY CREDITS"</span>
                        </div>
                        <div class="pack-list">
                            <div 
                                class=move || if selected_pack.get() == 5 { "pack-item active" } else { "pack-item" }
                                on:click=move |_| set_selected_pack.set(5)
                            >
                                <div class="pack-info">
                                    <span class="pack-name">"Basic Pack"</span>
                                    <span class="pack-credits">"35 CREDITS"</span>
                                </div>
                                <span class="pack-price">"5€"</span>
                            </div>
                            <div 
                                class=move || if selected_pack.get() == 10 { "pack-item active" } else { "pack-item" }
                                on:click=move |_| set_selected_pack.set(10)
                            >
                                <div class="pack-info">
                                    <span class="pack-name">"Studio Pack"</span>
                                    <span class="pack-credits">"80 CREDITS"</span>
                                </div>
                                <span class="pack-price">"10€"</span>
                            </div>
                        </div>
                        
                        <div class="card-actions-row centered-actions" style="display: flex; justify-content: center; margin-top: var(--s-10);">
                            <button 
                                class="btn btn-primary btn-lg" 
                                style="width: 65%; font-size: 0.8125rem; font-weight: 800; padding: var(--s-4) 0;"
                                class:loading=loading
                                on:click=on_buy
                                disabled=loading
                            >
                                {move || if loading.get() { "STARTING CHECKOUT..." } else { "BUY CREDITS" }}
                            </button>
                        </div>
                        
                        <div class="legal-disclosure" style="margin-top: var(--s-6); font-size: 0.625rem; color: hsl(var(--text-dim) / 0.7); line-height: 1.5; text-align: center; max-width: 90%; margin-left: auto; margin-right: auto;">
                            "By clicking BUY CREDITS, you agree to our "
                            <a href="/terms" style="color: inherit; text-decoration: underline;">"Terms"</a> " and "
                            <a href="/refunds" style="color: inherit; text-decoration: underline;">"Refund Policy"</a>". "
                            "You consent to immediate performance and acknowledge that you lose your right of withdrawal once you begin using any credits."
                        </div>
                    </div>
                </div>
            </div>

            <div class="history-section">
                <div class="history-header">
                    <div class="history-title">
                        <h2>"Logs"</h2>
                        <p class="muted">"History of your previous upscales and credits usage."</p>
                    </div>
                    <div class="telemetry-badge">
                        <span class="badge-label">"UPSCALED IMAGES"</span>
                        <span class="badge-value"> {move || auth.history.get().map(|v| v.len().to_string()).unwrap_or_else(|| "0".to_string())}</span>
                    </div>
                </div>
                
                <div class="usage-card">
                    <div class="table-wrapper">
                        <table class="usage-table">
                                    <thead>
                                        <tr>
                                            <th>"ID"</th>
                                            <th>"TIMESTAMP"</th>
                                            <th class="text-center">"QUALITY"</th>
                                            <th class="text-center">"STYLE"</th>
                                            <th class="text-center">"CREDITS"</th>
                                            <th class="text-center">"STATUS"</th>
                                            <th class="text-right">"TIME"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <Suspense fallback=|| view! { <tr><td colspan="7" style="padding: 6rem; text-align: center; opacity: 0.3;">"Synchronizing telemetry stream..."</td></tr> }>
                                            {move || {
                                                let h = auth.history.get();
                                                match h {
                                                    Some(items) => items.into_iter().map(|item| {
                                                        let id_short = item.id.to_string()[..8].to_string().to_uppercase();
                                                        let status_label = if item.status == "COMPLETED" { "SUCCESS".to_string() } else { item.status.clone() };
                                                        let item_url = item.image_url;
                                                        
                                                        // Format ISO timestamp: "2026-04-21T11:17:41..." -> "Apr 21, 11:17"
                                                        let raw_ts = item.created_at.clone();
                                                        let formatted_ts = if raw_ts.len() > 16 {
                                                            let parts: Vec<&str> = raw_ts.split('T').collect();
                                                            let date_p: Vec<&str> = parts[0].split('-').collect();
                                                            let time_p: Vec<&str> = parts[1].split(':').collect();
                                                            if date_p.len() >= 3 && time_p.len() >= 2 {
                                                                let month = match date_p[1] {
                                                                    "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
                                                                    "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
                                                                    "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
                                                                    _ => date_p[1]
                                                                };
                                                                format!("{} {}, {}:{}", month, date_p[2], time_p[0], time_p[1])
                                                            } else { raw_ts }
                                                        } else { raw_ts };
                                                        
                                                        let item_quality = item.quality.replace(" RECON", "");
                                                        let item_style = item.style.unwrap_or_else(|| "AUTO".to_string());
                                                        let item_status_lower = item.status.to_lowercase();
                                                        let item_latency = format!("{:.1}S", item.latency_ms as f32 / 1000.0);
                                                        let item_credits = format!("{}C", item.credits_charged);
                                                        
                                                        view! {
                                                            <tr>
                                                                <td class="id-cell">
                                                                    {match item_url {
                                                                        Some(url) => view! { <a href=url target="_blank" class="cell-link">{id_short}</a> }.into_any(),
                                                                        None => view! { <span class="dim">{id_short}</span> }.into_any(),
                                                                    }}
                                                                </td>
                                                                <td style="color: hsl(var(--text-dim))">{formatted_ts}</td>
                                                                <td class="text-center">{item_quality}</td>
                                                                <td class="text-center">{item_style}</td>
                                                                <td class="text-center" style="font-weight: 800; color: hsl(var(--accent))">{item_credits}</td>
                                                                <td class="text-center"><span class=format!("status-chip {}", item_status_lower)>{status_label}</span></td>
                                                                <td class="text-right" style="color: hsl(var(--success)); font-weight: 800;">{item_latency}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any(),
                                                    None => view! { <tr><td colspan="7" style="padding: 6rem; text-align: center; opacity: 0.3;">"Acquiring telemetry data..."</td></tr> }.into_any()
                                                }
                                            }}
                                        </Suspense>
                                    </tbody>
                        </table>
                    </div>
                </div>
            </div>


        </div>
    }
}

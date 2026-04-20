use leptos::prelude::*;
use crate::components::icons::{Zap, ShieldCheck, Info, HistoryIcon};
use crate::auth::use_auth;
use crate::api::ApiClient;

#[component]
pub fn Credits() -> impl IntoView {
    let auth = use_auth();
    
    // Fetch balance if not already present in global signal
    Effect::new(move |_| {
        if auth.credits.get().is_none() {
            let token = auth.session.get().map(|s| s.access_token);
            leptos::task::spawn_local(async move {
                if let Ok(bal) = ApiClient::get_balance(token.as_deref()).await {
                    auth.set_credits.set(Some(bal));
                }
            });
        }
    });

    let history_count = LocalResource::new(move || {
        let token = auth.session.get().map(|s| s.access_token);
        async move {
            ApiClient::get_history(token.as_deref()).await.map(|h| h.len()).unwrap_or(0)
        }
    });

    view! {
        <div class="credits-container fade-in">
            <div class="page-header">
                <h1>"Capacity & Statistics"</h1>
                <p class="muted">"Manage your upscaling throughput and resource balance."</p>
            </div>

            <div class="credits-grid">
                <div class="card balance-card">
                    <div class="card-body">
                        <div class="balance-meta">
                            <Zap size={20} />
                            <span>"Available Capacity"</span>
                        </div>
                        
                        <div class="balance-main">
                            <span class="credits-count">
                                {move || auth.credits.get().map(|c| c.to_string()).unwrap_or_else(|| "---".to_string())}
                            </span>
                            <span class="credits-unit">"UNITS"</span>
                        </div>

                        <div class="balance-actions">
                            <button class="btn btn-primary btn-lg" style="flex: 1;" on:click=move |_| {
                                leptos::logging::log!("Stripe checkout would open here");
                            }>"RECHARGE PIPELINE"</button>
                            <button class="btn btn-secondary">"LOGS"</button>
                        </div>
                    </div>
                </div>

                <div class="stats-sidebar">
                    <div class="card stat-mini-card">
                        <div class="card-header">
                            <HistoryIcon size={16} />
                            <span>"Activity"</span>
                        </div>
                        <div class="stat-mini-body">
                            <div class="mini-stat-item">
                                <span class="label">"TOTAL PROJECTS"</span>
                                <Suspense fallback=|| view! { <span class="value">"..."</span> }>
                                    <span class="value">{move || history_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}</span>
                                </Suspense>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"PIPELINE VERSION"</span>
                                <span class="value">"V7.1 STABLE"</span>
                            </div>
                        </div>
                    </div>

                    <div class="card pricing-card">
                        <div class="card-header">
                            <Info size={16} />
                            <span>"Protocol Costs"</span>
                        </div>
                        <div class="pricing-list">
                            <div class="price-item">
                                <span class="p-label">"2K RECONSTRUCTION"</span>
                                <span class="p-value">"2 CREDITS"</span>
                            </div>
                            <div class="price-item">
                                <span class="p-label">"4K RECONSTRUCTION"</span>
                                <span class="p-value">"4 CREDITS"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="security-alert">
                <ShieldCheck size={18} />
                <span>"Transactions are verified via encrypted Stripe infrastructure protocol."</span>
            </div>

            <style>
                ".credits-container { max-width: 1000px; margin: 0 auto; }
                .page-header { margin-bottom: 3rem; }
                .page-header h1 { font-size: 2.25rem; font-weight: 800; letter-spacing: -0.04em; }

                .credits-grid { display: grid; grid-template-columns: 1.2fr 1fr; gap: 2rem; align-items: flex-start; }
                
                .balance-card .card-body { padding: 3rem; display: flex; flex-direction: column; gap: 2rem; }
                .balance-meta { display: flex; align-items: center; gap: 0.75rem; color: var(--accent); font-weight: 800; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.1em; }
                .balance-main { display: flex; align-items: baseline; gap: 1rem; padding: 1rem 0; border-bottom: 1px solid var(--border-color); }
                .credits-count { font-size: 5rem; font-weight: 800; line-height: 1; letter-spacing: -0.05em; font-family: var(--font-mono); }
                .credits-unit { font-size: 0.8rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.1em; }
                
                .balance-actions { display: flex; gap: 1rem; margin-top: 1rem; }
                
                .stats-sidebar { display: flex; flex-direction: column; gap: 1.5rem; }
                .stat-mini-body { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; }
                .mini-stat-item { display: flex; justify-content: space-between; align-items: center; }
                .mini-stat-item .label { font-size: 0.6rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
                .mini-stat-item .value { font-size: 1.25rem; font-weight: 700; font-family: var(--font-mono); }
                
                .pricing-list { padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; }
                .price-item { display: flex; justify-content: space-between; align-items: center; }
                .p-label { font-size: 0.65rem; font-weight: 700; color: var(--text-muted); }
                .p-value { font-size: 0.75rem; font-weight: 700; font-family: var(--font-mono); color: var(--accent); }
                
                .security-alert { margin-top: 3rem; padding: 1.5rem; border-radius: 8px; border: 1px solid var(--border-color); display: flex; align-items: center; gap: 1rem; font-size: 0.7rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.02em; }
                
                @media (max-width: 850px) {
                    .credits-grid { grid-template-columns: 1fr; }
                }
                "
            </style>
        </div>
    }
}

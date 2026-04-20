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
                <h1 class="text-gradient">"Infrastructure Capacity"</h1>
                <p class="muted">"Manage your upscaling throughput and resource allocation."</p>
            </div>

            <div class="credits-grid">
                <div class="card balance-card">
                    <div class="card-body">
                        <div class="balance-meta">
                            <Zap size={14} />
                            <span>"AVAILABLE UNITS"</span>
                        </div>
                        
                        <div class="balance-main">
                            <span class="credits-count">
                                {move || auth.credits.get().map(|c| c.to_string()).unwrap_or_else(|| "---".to_string())}
                            </span>
                        </div>

                        <div class="balance-actions">
                            <button class="btn btn-primary btn-lg" style="width: 100%;" on:click=move |_| {
                                leptos::logging::log!("Stripe checkout would open here");
                            }>"BUY CREDITS"</button>
                            <button class="btn btn-secondary btn-block">"VIEW USAGE LOGS"</button>
                        </div>
                    </div>
                </div>

                <div class="stats-sidebar">
                    <div class="card stat-mini-card">
                        <div class="card-header">
                            <HistoryIcon size={16} />
                            <span>"PIPELINE METRICS"</span>
                        </div>
                        <div class="stat-mini-body">
                            <div class="mini-stat-item">
                                <span class="label">"TOTAL PROJECTS"</span>
                                <Suspense fallback=|| view! { <span class="value">"..."</span> }>
                                    <span class="value">{move || history_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}</span>
                                </Suspense>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"AVG LATENCY"</span>
                                <span class="value">"~15S"</span>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"ARCHITECTURE"</span>
                                <span class="value">"V7.1 STABLE"</span>
                            </div>
                        </div>
                    </div>

                    <div class="card pricing-mini-card" style="margin-top: 1.5rem;">
                        <div class="card-header">
                            <Info size={16} />
                            <span>"UNIT PROTOCOL"</span>
                        </div>
                        <div class="pricing-list" style="padding: 1.5rem; display: flex; flex-direction: column; gap: 0.75rem;">
                            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; font-weight: 600;">
                                <span class="muted">"2K RECON"</span>
                                <span class="accent">"2 UNITS"</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; font-weight: 600;">
                                <span class="muted">"4K RECON"</span>
                                <span class="accent">"4 UNITS"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="security-note">
                <ShieldCheck size={16} />
                <span>"TRANSACTIONS ARE VERIFIED VIA STRIPE INFRASTRUCTURE PROTOCOLS."</span>
            </div>

            <style>
                ".credits-container { max-width: 900px; margin: 0 auto; text-align: left; }
                .credits-grid { display: grid; grid-template-columns: 1.3fr 1fr; gap: 2rem; align-items: stretch; margin-top: 3rem; }
                
                .balance-card .card-body { padding: 3rem 2.5rem; display: flex; flex-direction: column; height: 100%; background: var(--surface-color); }
                .balance-meta { display: flex; align-items: center; gap: 0.6rem; color: var(--accent); font-weight: 800; font-size: 0.6rem; letter-spacing: 0.1em; margin-bottom: 1rem; }
                .balance-main { margin-bottom: 3rem; padding-bottom: 1rem; border-bottom: 1px solid var(--border-color); }
                .credits-count { font-size: 5.5rem; font-weight: 800; line-height: 1; letter-spacing: -0.04em; font-family: var(--font-mono); color: var(--text-color); }
                
                .balance-actions { display: flex; flex-direction: column; gap: 1rem; margin-top: auto; }
                .btn-block { width: 100%; border-color: var(--border-color); }
                
                .stat-mini-body { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; }
                .mini-stat-item { display: flex; justify-content: space-between; align-items: center; }
                .mini-stat-item .label { font-size: 0.6rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
                .mini-stat-item .value { font-size: 1rem; font-weight: 700; font-family: var(--font-mono); color: var(--text-color); }
                
                .accent { color: var(--accent); }
                
                .security-note { margin-top: 3rem; color: var(--text-muted); font-size: 0.65rem; font-weight: 700; letter-spacing: 0.05em; display: flex; align-items: center; gap: 0.75rem; justify-content: center; opacity: 0.6; }
                
                @media (max-width: 850px) {
                    .credits-grid { grid-template-columns: 1fr; gap: 1.5rem; }
                    .credits-count { font-size: 4rem; }
                    .balance-card .card-body { padding: 2.5rem 1.5rem; }
                    .page-header h1 { font-size: 1.75rem; }
                }
                "
            </style>
        </div>
    }
}

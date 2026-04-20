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
                        <div class="mini-stat-header">
                            <HistoryIcon size={14} />
                            <span>"PIPELINE METRICS"</span>
                        </div>
                        <div class="stat-mini-body">
                            <div class="mini-stat-item">
                                <span class="label">"TOTAL PROJECTS"</span>
                                <Suspense fallback=|| view! { <span class="value">"---"</span> }>
                                    <span class="value">{move || history_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}</span>
                                </Suspense>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"AVG LATENCY"</span>
                                <span class="value">"~15S"</span>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"ARCHITECTURE"</span>
                                <span class="value">"V1.0 ALPHA"</span>
                            </div>
                        </div>
                    </div>

                    <div class="card pricing-mini-card">
                        <div class="mini-stat-header">
                            <Info size={14} />
                            <span>"UNIT PROTOCOL"</span>
                        </div>
                        <div class="pricing-list">
                            <div class="pricing-row">
                                <span class="label">"2K RECON"</span>
                                <span class="value">"2 UNITS"</span>
                            </div>
                            <div class="pricing-row">
                                <span class="label">"4K RECON"</span>
                                <span class="value">"4 UNITS"</span>
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
                ".credits-container { width: 100%; max-width: 1200px; margin: 0 auto; }
                .credits-grid { display: grid; grid-template-columns: 1.3fr 1fr; gap: var(--s-8); margin-top: var(--s-12); align-items: stretch; }
                
                .balance-card .card-body { padding: var(--s-12); display: flex; flex-direction: column; height: 100%; gap: var(--s-8); background: radial-gradient(circle at top right, hsl(var(--accent) / 0.05), transparent); }
                .balance-meta { display: flex; align-items: center; gap: var(--s-3); color: hsl(var(--accent)); font-weight: 800; font-size: 0.625rem; letter-spacing: 0.2em; text-transform: uppercase; }
                .balance-main { padding-bottom: var(--s-8); border-bottom: 1px solid var(--glass-border); display: flex; align-items: baseline; gap: var(--s-4); position: relative; }
                
                .credits-count { 
                    font-size: clamp(4rem, 10vw, 6.5rem); 
                    font-weight: 800; 
                    line-height: 0.85; 
                    letter-spacing: -0.06em; 
                    font-family: var(--font-mono); 
                    color: hsl(var(--text));
                    background: linear-gradient(90deg, hsl(var(--text)) 0%, hsl(var(--text-muted)) 50%, hsl(var(--text)) 100%);
                    background-size: 200% auto;
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    animation: shimmer 5s linear infinite;
                }
                
                .balance-actions { display: flex; flex-direction: column; gap: var(--s-3); margin-top: auto; }
                .btn-block { width: 100%; }
                
                .stats-sidebar { display: flex; flex-direction: column; gap: var(--s-6); }
                .stat-mini-card, .pricing-mini-card { background: hsl(var(--surface-raised) / 0.5); }
                
                .mini-stat-header { padding: var(--s-4) var(--s-6); border-bottom: 1px solid var(--glass-border); display: flex; align-items: center; gap: var(--s-3); font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-muted)); letter-spacing: 0.15em; text-transform: uppercase; }
                .stat-mini-body { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-6); }
                .mini-stat-item { display: flex; justify-content: space-between; align-items: center; }
                .mini-stat-item .label { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .mini-stat-item .value { font-size: 0.875rem; font-weight: 700; font-family: var(--font-mono); color: hsl(var(--text)); text-shadow: 0 0 10px rgba(255,255,255,0.1); }
                
                .pricing-list { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-4); }
                .pricing-row { display: flex; justify-content: space-between; font-size: 0.75rem; font-weight: 700; }
                .pricing-row .label { color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; font-size: 0.625rem; }
                .pricing-row .value { color: hsl(var(--accent)); font-family: var(--font-mono); }
                
                .security-note { margin-top: var(--s-16); color: hsl(var(--text-dim)); font-size: 0.625rem; font-weight: 800; letter-spacing: 0.15em; display: flex; align-items: center; gap: var(--s-4); justify-content: center; opacity: 0.6; text-transform: uppercase; border: 1px solid var(--glass-border); padding: var(--s-2) var(--s-6); border-radius: 100px; width: fit-content; margin-left: auto; margin-right: auto; }
                
                @media (max-width: 850px) {
                    .credits-grid { grid-template-columns: 1fr; gap: var(--s-6); }
                    .credits-count { font-size: 4rem; }
                    .balance-card .card-body { padding: var(--s-8) var(--s-6); }
                }
                "
            </style>
        </div>
    }
}

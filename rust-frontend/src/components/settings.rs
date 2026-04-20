use leptos::prelude::*;
use crate::components::icons::{Zap, ShieldCheck, Info};
use crate::auth::use_auth;
use crate::api::ApiClient;

#[component]
pub fn Credits() -> impl IntoView {
    let auth = use_auth();
    let balance = LocalResource::new(
        move || { 
            let token = auth.session.get().map(|s| s.access_token);
            async move {
                ApiClient::get_balance(token.as_deref()).await
            }
        }
    );

    view! {
        <div class="credits-container fade-in">
            <div class="page-header">
                <h1>"Credits & Billing"</h1>
                <p class="muted">"Manage your upscaling capacity and project balance."</p>
            </div>

            <div class="credits-grid">
                {/* Credit Overview */}
                <div class="card balance-card">
                    <div class="card-body">
                        <div class="balance-meta">
                            <Zap size={20} />
                            <span>"Current Availability"</span>
                        </div>
                        
                        <div class="balance-main">
                            <Suspense fallback=|| view! { <div class="skeleton-text" style="width: 80px; height: 60px;"></div> }>
                                {move || Suspend::new(async move {
                                    let res = balance.await;
                                    match res {
                                        Ok(credits) => view! { <span class="credits-count">{credits}</span> }.into_any(),
                                        _ => view! { <span class="credits-count error">"--"</span> }.into_any(),
                                    }
                                })}
                            </Suspense>
                            <span class="credits-unit">"REMAINING"</span>
                        </div>

                        <div class="balance-actions">
                            <button class="btn btn-primary btn-lg" style="flex: 1;">"RECHARGE CREDITS"</button>
                            <button class="btn btn-secondary">"TRANSACTION LOG"</button>
                        </div>
                    </div>
                </div>

                {/* Info Card */}
                <div class="card info-card">
                    <div class="card-body">
                        <h3>"Upscaling Tiers"</h3>
                        <p class="muted" style="font-size: 0.8rem; margin-bottom: 1.5rem;">"Our pricing is transparent and based on output fidelity."</p>
                        
                        <div class="pricing-list">
                            <div class="price-item">
                                <span class="label">"2K (High Fidelity)"</span>
                                <span class="value">"2 CREDITS"</span>
                            </div>
                            <div class="price-item">
                                <span class="label">"4K (Ultra Fidelity)"</span>
                                <span class="value">"4 CREDITS"</span>
                            </div>
                            <div class="price-item highlight">
                                <span class="label">"Bulk Acquisition"</span>
                                <span class="value">"SAVE 20%"</span>
                            </div>
                        </div>

                        <div class="security-note">
                            <ShieldCheck size={14} />
                            <span>"Payments secured via encrypted pipeline"</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="support-alert">
                <Info size={18} />
                <div class="alert-content">
                    <h4>"Enterprise & API"</h4>
                    <p>"For high-volume automation or dedicated infrastructure, please contact our technical sales team."</p>
                </div>
            </div>

            <style>
                ".credits-container { max-width: 900px; margin: 0 auto; }
                .page-header { margin-bottom: 4rem; text-align: left; }
                .page-header h1 { font-size: 2.25rem; font-weight: 800; letter-spacing: -0.04em; }

                .credits-grid { display: grid; grid-template-columns: 1.2fr 1fr; gap: 2rem; }
                .balance-card { padding: 0.5rem; border-color: var(--border-color); }
                .balance-card .card-body { padding: 2.5rem; display: flex; flex-direction: column; gap: 2rem; }
                
                .balance-meta { display: flex; align-items: center; gap: 0.75rem; color: var(--accent); font-weight: 700; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.05em; }
                .balance-main { display: flex; align-items: baseline; gap: 1rem; padding: 1.5rem 0; border-bottom: 1px solid var(--border-color); }
                .credits-count { font-size: 4rem; font-weight: 800; line-height: 1; letter-spacing: -0.05em; font-family: var(--font-mono); }
                .credits-unit { font-size: 0.8rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
                .credits-count.error { color: var(--error); }

                .balance-actions { display: flex; gap: 1rem; margin-top: 1rem; }

                .info-card .card-body { padding: 2.5rem; }
                .info-card h3 { font-size: 1.125rem; font-weight: 700; margin-bottom: 0.5rem; }
                
                .pricing-list { display: flex; flex-direction: column; gap: 1rem; }
                .price-item { display: flex; justify-content: space-between; padding-bottom: 0.75rem; border-bottom: 1px solid var(--border-color); }
                .price-item .label { font-size: 0.75rem; font-weight: 600; color: var(--text-muted); }
                .price-item .value { font-size: 0.75rem; font-weight: 700; font-family: var(--font-mono); }
                .price-item.highlight .value { color: var(--success); }

                .security-note { margin-top: 2rem; display: flex; align-items: center; gap: 0.5rem; font-size: 0.65rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }

                .support-alert { margin-top: 3rem; padding: 2rem; border-radius: 8px; background: var(--surface-color); border: 1px solid var(--border-color); display: flex; gap: 1.5rem; align-items: flex-start; }
                .alert-content h4 { font-size: 0.9rem; font-weight: 700; margin-bottom: 0.25rem; }
                .alert-content p { font-size: 0.85rem; color: var(--text-muted); }
                
                @media (max-width: 800px) {
                    .credits-grid { grid-template-columns: 1fr; }
                }
                "
            </style>
        </div>
    }
}

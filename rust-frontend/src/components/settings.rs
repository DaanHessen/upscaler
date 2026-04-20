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
            let auth_ctx = auth.clone();
            leptos::task::spawn_local(async move {
                match ApiClient::get_balance(token.as_deref()).await {
                    Ok(bal) => auth_ctx.set_credits.set(Some(bal)),
                    Err(e) if e == "AUTH_EXPIRED" => auth_ctx.logout(),
                    Err(_) => {}
                }
            });
        }
    });

    let history_data = LocalResource::new(move || {
        let token = auth.session.get().map(|s| s.access_token);
        async move {
            ApiClient::get_history(token.as_deref()).await
        }
    });

    view! {
        <div class="credits-container fade_in">
            <div class="page-header">
                <h1 class="text-gradient">"Credits & Usage"</h1>
                <p class="muted">"Manage your upscaling credits and view your activity history."</p>
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
                        </div>
                    </div>
                </div>

                <div class="stats-sidebar">
                    <div class="card pricing-mini-card">
                        <div class="mini-stat-header">
                            <Zap size={14} />
                            <span>"CHOOSE YOUR PACK"</span>
                        </div>
                        <div class="pricing-grid">
                            <div class="pricing-card">
                                <span class="p-name">"STARTER"</span>
                                <span class="p-units">"10 UNITS"</span>
                                <button class="btn btn-primary btn-sm">"$5"</button>
                            </div>
                            <div class="pricing-card featured">
                                <span class="p-name">"STUDIO"</span>
                                <span class="p-units">"50 UNITS"</span>
                                <button class="btn btn-primary btn-sm">"$20"</button>
                            </div>
                            <div class="pricing-card">
                                <span class="p-name">"ENTERPRISE"</span>
                                <span class="p-units">"250 UNITS"</span>
                                <button class="btn btn-primary btn-sm">"$75"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="usage-section">
                <div class="section-header">
                    <h2 class="text-gradient">"Upscale History"</h2>
                    <p class="muted">"Review your recent upscaling activity."</p>
                </div>
                
                <div class="card usage-card">
                    <div class="table-wrapper">
                        <table class="usage-table">
                            <thead>
                                <tr>
                                    <th>"RESULT"</th>
                                    <th>"DATE"</th>
                                    <th>"RESOLUTION"</th>
                                    <th>"STYLE"</th>
                                    <th>"STATUS"</th>
                                    <th>"TIME"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <Suspense fallback=|| view! { <tr><td colspan="6" class="placeholder">"Fetching telemetry..."</td></tr> }>
                                    {move || history_data.get().map(|res| {
                                        let auth_ctx = auth.clone();
                                        match (*res).clone() {
                                            Ok(items) => items.into_iter().map(|item| {
                                                let id_short = item.id.to_string()[..8].to_string();
                                                let status_label = if item.status == "COMPLETED" { "VERIFIED".to_string() } else { item.status.clone() };
                                                let item_url = item.image_url;
                                                let item_created = item.created_at;
                                                let item_quality = item.quality.replace(" RECON", "");
                                                let item_style = item.style.unwrap_or_else(|| "AUTO".to_string());
                                                let item_status_lower = item.status.to_lowercase();
                                                
                                                view! {
                                                    <tr>
                                                        <td class="mono">
                                                            {match item_url {
                                                                Some(url) => view! { <a href=url target="_blank" class="result-link">"VIEW RESULT"</a> }.into_any(),
                                                                None => view! { <span class="dim">{id_short}</span> }.into_any(),
                                                            }}
                                                        </td>
                                                        <td class="mono">{item_created}</td>
                                                        <td>{item_quality}</td>
                                                        <td>{item_style}</td>
                                                        <td><span class=format!("status-tag status-{}", item_status_lower)>{status_label}</span></td>
                                                        <td class="mono dim">"~15S"</td>
                                                    </tr>
                                                }
                                            }).collect_view().into_any(),
                                            Err(e) if e == "AUTH_EXPIRED" => {
                                                auth_ctx.logout();
                                                view! { <tr><td colspan="6" class="error">"Session expired. Logging out..."</td></tr> }.into_any()
                                            },
                                            Err(_) => view! { <tr><td colspan="6" class="error">"Telemetry unavailable"</td></tr> }.into_any()
                                        }
                                    })}
                                </Suspense>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="security-note">
                <ShieldCheck size={16} />
                <span>"TRANSACTIONS ARE VERIFIED VIA STRIPE INFRASTRUCTURE PROTOCOLS."</span>
            </div>

            <style>
                ".credits-container { width: 100%; max-width: 1200px; margin: 0 auto; padding-bottom: var(--s-20); }
                .credits-grid { display: grid; grid-template-columns: 1.3fr 1fr; gap: var(--s-8); margin-top: var(--s-12); align-items: stretch; }
                
                .balance-card .card-body { padding: var(--s-12); display: flex; flex-direction: column; height: 100%; gap: var(--s-8); background: radial-gradient(circle at top right, hsl(var(--accent) / 0.05), transparent); }
                .balance-meta { display: flex; align-items: center; gap: var(--s-3); color: hsl(var(--accent)); font-weight: 800; font-size: 0.625rem; letter-spacing: 0.2em; text-transform: uppercase; }
                .balance-main { padding-bottom: var(--s-8); border-bottom: 1px solid var(--glass-border); display: flex; align-items: baseline; gap: var(--s-4); position: relative; }
                
                .credits-count { 
                    font-size: clamp(3rem, 12vw, 5.5rem); 
                    font-weight: 800; 
                    line-height: 1; 
                    letter-spacing: -0.04em; 
                    font-family: var(--font-mono); 
                    color: hsl(var(--text));
                    background: linear-gradient(90deg, hsl(var(--text)) 0%, hsl(var(--text-muted)) 50%, hsl(var(--text)) 100%);
                    background-size: 200% auto;
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    animation: shimmer 5s linear infinite;
                }
                
                .balance-actions { display: flex; flex-direction: column; gap: var(--s-3); margin-top: auto; padding-top: var(--s-4); }
                
                .stats-sidebar { display: flex; flex-direction: column; gap: var(--s-6); }
                .stat-mini-card, .pricing-mini-card { background: hsl(var(--surface-raised) / 0.5); overflow: hidden; }
                
                .pricing-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--s-4); padding: var(--s-6); }
                .pricing-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: var(--s-4); display: flex; flex-direction: column; align-items: center; gap: var(--s-2); transition: all 0.2s; }
                .pricing-card:hover { border-color: hsl(var(--accent) / 0.4); transform: translateY(-2px); }
                .pricing-card.featured { border-color: hsl(var(--accent)); background: hsl(var(--accent) / 0.05); }
                .p-name { font-size: 0.5rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .p-units { font-size: 0.75rem; font-weight: 900; color: hsl(var(--text)); font-family: var(--font-mono); }
                
                .mini-stat-header { padding: var(--s-4) var(--s-6); border-bottom: 1px solid var(--glass-border); display: flex; align-items: center; gap: var(--s-3); font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-muted)); letter-spacing: 0.15em; text-transform: uppercase; }
                .stat-mini-body { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-6); }
                .mini-stat-item { display: flex; justify-content: space-between; align-items: center; }
                .mini-stat-item .label { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .mini-stat-item .value { font-size: 0.875rem; font-weight: 700; font-family: var(--font-mono); color: hsl(var(--text)); text-shadow: 0 0 10px rgba(255,255,255,0.1); }
                
                .pricing-list { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-4); }
                .pricing-row { display: flex; justify-content: space-between; font-size: 0.75rem; font-weight: 700; }
                .pricing-row .label { color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; font-size: 0.625rem; }
                .pricing-row .value { color: hsl(var(--accent)); font-family: var(--font-mono); }

                .usage-section { margin-top: var(--s-20); }
                .section-header { margin-bottom: var(--s-8); }
                .usage-card { background: hsl(var(--surface) / 0.3); padding: 1px; overflow: hidden; }
                .table-wrapper { width: 100%; overflow-x: auto; }
                .usage-table { width: 100%; border-collapse: collapse; font-size: 0.75rem; text-align: left; }
                .usage-table th { padding: var(--s-4) var(--s-6); color: hsl(var(--text-dim)); font-weight: 800; letter-spacing: 0.1em; border-bottom: 1px solid var(--glass-border); text-transform: uppercase; font-size: 0.625rem; }
                .usage-table td { padding: var(--s-4) var(--s-6); border-bottom: 1px solid var(--glass-border) / 0.5; color: hsl(var(--text-muted)); font-weight: 500; }
                .usage-table tr:last-child td { border-bottom: none; }
                .usage-table tr:hover td { background: hsl(var(--accent) / 0.02); color: hsl(var(--text)); }
                
                .mono { font-family: var(--font-mono); font-size: 0.7rem; }
                .dim { color: hsl(var(--text-dim)); }
                .result-link { color: hsl(var(--accent)); text-decoration: none; font-weight: 800; }
                .result-link:hover { text-decoration: underline; }
                
                .status-tag { font-size: 0.5rem; font-weight: 900; padding: 2px 6px; border-radius: 4px; border: 1px solid currentColor; letter-spacing: 0.05em; }
                .status-completed { color: hsl(var(--success)); border-color: hsl(var(--success) / 0.3); background: hsl(var(--success) / 0.05); }
                .status-processing { color: hsl(var(--accent)); border-color: hsl(var(--accent) / 0.3); background: hsl(var(--accent) / 0.05); }
                .status-failed { color: hsl(var(--error)); border-color: hsl(var(--error) / 0.3); background: hsl(var(--error) / 0.05); }

                .security-note { margin-top: var(--s-16); color: hsl(var(--text-dim)); font-size: 0.625rem; font-weight: 800; letter-spacing: 0.15em; display: flex; align-items: center; gap: var(--s-4); justify-content: center; opacity: 0.6; text-transform: uppercase; border: 1px solid var(--glass-border); padding: var(--s-2) var(--s-6); border-radius: 100px; width: fit-content; margin-left: auto; margin-right: auto; }
                
                @media (max-width: 850px) {
                    .credits-grid { grid-template-columns: 1fr; gap: var(--s-6); }
                    .credits-count { font-size: 4rem; }
                    .usage-section { margin-top: var(--s-12); }
                }
                "
            </style>
        </div>
    }
}

use leptos::prelude::*;
use crate::components::icons::{Zap, ShieldCheck, HistoryIcon};
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
                            <span>"RECONSTRUCTION BALANCE"</span>
                        </div>
                        
                        <div class="balance-main">
                            <div class="count-box">
                                <span class="credits-count">
                                    {move || auth.credits.get().map(|c| c.to_string()).unwrap_or_else(|| "---".to_string())}
                                </span>
                                <span class="unit-label">"UNITS AVAILABLE"</span>
                            </div>
                        </div>

                        <div class="balance-details">
                            <div class="detail-row">
                                <span class="d-label">"ACCOUNT STATUS"</span>
                                <span class="d-value success">"ACTIVE"</span>
                            </div>
                            <div class="detail-row">
                                <span class="d-label">"LAST TOP UP"</span>
                                <span class="d-value">"NO RECENT ACTIVITY"</span>
                            </div>
                        </div>

                        <div class="pricing-options">
                            <button class="pricing-btn" on:click=move |_| {}>
                                <div class="p-info">
                                    <span class="p-title">"BASIC PACK"</span>
                                    <span class="p-desc">"Perfect for quick restoration"</span>
                                </div>
                                <div class="p-cost">
                                    <span class="p-price">"5€"</span>
                                    <span class="p-qty">"35 UNITS"</span>
                                </div>
                            </button>
                            <button class="pricing-btn featured" on:click=move |_| {}>
                                <div class="p-info">
                                    <span class="p-title">"STUDIO PACK"</span>
                                    <span class="p-desc">"Our most popular bundle"</span>
                                </div>
                                <div class="p-cost">
                                    <span class="p-price">"10€"</span>
                                    <span class="p-qty">"80 UNITS"</span>
                                </div>
                            </button>
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
                                    <span class="value">{move || history_data.get().and_then(|res| (*res).as_ref().ok().map(|v| v.len().to_string())).unwrap_or_else(|| "0".to_string())}</span>
                                </Suspense>
                            </div>
                            <div class="mini-stat-item">
                                <span class="label">"ARCHITECTURE"</span>
                                <span class="value">"STUDIO V1.0"</span>
                            </div>
                        </div>
                    </div>

                    <div class="card stat-mini-card">
                        <div class="mini-stat-header">
                            <ShieldCheck size={14} />
                            <span>"DATA PROTOCOL"</span>
                        </div>
                        <div class="stat-mini-body">
                            <p class="sidebar-text">"All transactions and upscales are encrypted via industry-standard protocols. Assets are purged after 24 hours."</p>
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
                
                .balance-card .card-body { padding: var(--s-10); display: flex; flex-direction: column; height: 100%; gap: var(--s-8); background: radial-gradient(circle at top right, hsl(var(--accent) / 0.05), transparent); }
                .balance-meta { display: flex; align-items: center; gap: var(--s-3); color: hsl(var(--accent)); font-weight: 800; font-size: 0.625rem; letter-spacing: 0.2em; text-transform: uppercase; }
                .balance-main { padding-bottom: var(--s-10); border-bottom: 1px solid var(--glass-border); display: flex; align-items: center; justify-content: center; }
                
                .count-box { text-align: center; display: flex; flex-direction: column; align-items: center; }
                .credits-count { 
                    font-size: clamp(3rem, 12vw, 5rem); 
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
                .unit-label { font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim)); letter-spacing: 0.2em; text-transform: uppercase; margin-top: var(--s-4); }
                
                .balance-details { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-6); padding: var(--s-4) 0; }
                .detail-row { display: flex; flex-direction: column; gap: 2px; }
                .d-label { font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .d-value { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text)); }
                .d-value.success { color: hsl(var(--success)); }

                .pricing-options { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); margin-top: auto; }
                .pricing-btn { 
                    background: hsl(var(--surface-raised) / 0.4); 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-md); 
                    padding: var(--s-5); 
                    display: flex; 
                    justify-content: space-between; 
                    align-items: center; 
                    cursor: pointer; 
                    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
                    text-align: left;
                }
                .pricing-btn:hover { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--surface-raised) / 0.8); transform: translateY(-2px); }
                .pricing-btn.featured { border-color: hsl(var(--accent)); background: hsl(var(--accent) / 0.03); }
                .pricing-btn.featured:hover { background: hsl(var(--accent) / 0.08); }
                
                .p-info { display: flex; flex-direction: column; gap: 2px; }
                .p-title { font-size: 0.625rem; font-weight: 900; color: hsl(var(--text)); letter-spacing: 0.05em; }
                .p-desc { font-size: 0.55rem; color: hsl(var(--text-dim)); font-weight: 500; }
                
                .p-cost { text-align: right; display: flex; flex-direction: column; }
                .p-price { font-size: 0.875rem; font-weight: 900; color: hsl(var(--accent)); }
                .p-qty { font-size: 0.5rem; font-weight: 800; color: hsl(var(--text-dim)); letter-spacing: 0.05em; }

                .stats-sidebar { display: flex; flex-direction: column; gap: var(--s-6); }
                .stat-mini-card { background: hsl(var(--surface-raised) / 0.5); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); }
                .mini-stat-header { padding: var(--s-4) var(--s-6); border-bottom: 1px solid var(--glass-border); display: flex; align-items: center; gap: var(--s-3); font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-muted)); letter-spacing: 0.15em; text-transform: uppercase; }
                .stat-mini-body { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-6); }
                .mini-stat-item { display: flex; justify-content: space-between; align-items: center; }
                .mini-stat-item .label { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); letter-spacing: 0.1em; }
                .mini-stat-item .value { font-size: 0.875rem; font-weight: 700; font-family: var(--font-mono); color: hsl(var(--text)); }
                .sidebar-text { font-size: 0.7rem; color: hsl(var(--text-dim)); line-height: 1.6; font-weight: 500; }
                
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

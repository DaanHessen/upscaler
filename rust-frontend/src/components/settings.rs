use leptos::prelude::*;
use crate::components::icons::{Zap, CreditCard};
use crate::auth::use_auth;

#[component]
pub fn Credits() -> impl IntoView {
    let auth = use_auth();
    
    // Trigger throttled telemetry sync on mount
    Effect::new(move |_| {
        auth.sync_telemetry(false);
    });

    let (selected_pack, set_selected_pack) = signal(10); // Default to 10 euro pack

    view! {
        <div class="credits-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="text-gradient">"Credits & Usage"</h1>
                    <p class="muted">"Manage your upscaling credits and view your activity history."</p>
                </div>
            </div>

            <div class="credits-layout">
                /* Balance Hero */
                <div class="card vault-card shadow-lg">
                    <div class="params-body">
                        <div class="card-tag">
                            <Zap size={10} />
                            <span>"VAULT BALANCE"</span>
                        </div>
                        <div class="balance-display">
                            <span class="credit-count">
                                {move || auth.credits.get().map(|c| c.to_string()).unwrap_or_else(|| "---".to_string())}
                            </span>
                            <span class="credit-symbol">"Credits Available"</span>
                        </div>
                        
                        <div class="meta-stats">
                            <div class="stat-box">
                                <span class="stat-label">"Last Top Up"</span>
                                <span class="stat-value">"N/A"</span>
                            </div>
                            <div class="stat-box">
                                <span class="stat-label">"Status"</span>
                                <span class="stat-value highlight">"VERIFIED"</span>
                            </div>
                        </div>
                    </div>
                </div>

                /* Replenish Hero */
                <div class="card replenish-card shadow-lg">
                    <div class="params-body">
                        <div class="card-tag">
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
                        
                        <div class="card-actions-row">
                            <button class="btn btn-primary btn-lg btn-block" on:click=move |_| {}>
                                "BUY CREDITS"
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="history-section">
                <div class="history-header">
                    <div class="history-title">
                        <h2>"Telemetry Logs"</h2>
                        <p class="muted">"Secure logs of all system-level reconstruction activity."</p>
                    </div>
                    <div class="telemetry-badge">
                        <span class="badge-label">"LOGGED ENTRIES:"</span>
                        <span class="badge-value">{move || auth.history.get().map(|v| v.len().to_string()).unwrap_or_else(|| "0".to_string())}</span>
                    </div>
                </div>
                
                <div class="card usage-card">
                    <div class="table-wrapper">
                        <table class="usage-table">
                            <thead>
                                <tr>
                                    <th>"ASSET ID"</th>
                                    <th>"TIMESTAMP"</th>
                                    <th>"FIDELITY"</th>
                                    <th>"ENGINE"</th>
                                    <th>"STATUS"</th>
                                    <th>"LATENCY"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <Suspense fallback=|| view! { <tr><td colspan="6" style="padding: 6rem; text-align: center; opacity: 0.3;">"Synchronizing telemetry stream..."</td></tr> }>
                                    {move || {
                                        let h = auth.history.get();
                                        match h {
                                            Some(items) => items.into_iter().map(|item| {
                                                let id_short = item.id.to_string()[..8].to_string();
                                                let status_label = if item.status == "COMPLETED" { "VERIFIED".to_string() } else { item.status.clone() };
                                                let item_url = item.image_url;
                                                let item_created = item.created_at;
                                                let item_quality = item.quality.replace(" RECON", "");
                                                let item_style = item.style.unwrap_or_else(|| "AUTO".to_string());
                                                let item_status_lower = item.status.to_lowercase();
                                                
                                                view! {
                                                    <tr>
                                                        <td class="id-cell">
                                                            {match item_url {
                                                                Some(url) => view! { <a href=url target="_blank" class="cell-link">{id_short}</a> }.into_any(),
                                                                None => view! { <span class="dim">{id_short}</span> }.into_any(),
                                                            }}
                                                        </td>
                                                        <td>{item_created}</td>
                                                        <td>{item_quality}</td>
                                                        <td>{item_style}</td>
                                                        <td><span class=format!("status-chip {}", item_status_lower)>{status_label}</span></td>
                                                        <td class="latency-cell">"~15.4S"</td>
                                                    </tr>
                                                }
                                            }).collect_view().into_any(),
                                            None => view! { <tr><td colspan="6" style="padding: 6rem; text-align: center; opacity: 0.3;">"Acquiring telemetry data..."</td></tr> }.into_any()
                                        }
                                    }}
                                </Suspense>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>


            <style>
                ".credits-container { width: 100%; max-width: 1200px; margin: 0 auto; }
                .page-header { margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); display: flex; justify-content: space-between; align-items: flex-end; }
                
                .credits-layout { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-12); margin-top: var(--s-6); align-items: stretch; }
                
                /* Card Geometry */
                .params-body { padding: var(--s-10); height: 100%; display: flex; flex-direction: column; }
                .card-tag { display: flex; align-items: center; gap: var(--s-2); font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; margin-bottom: var(--s-8); opacity: 0.6; }
                
                .vault-card, .replenish-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); transition: border-color 0.3s; }
                .vault-card:hover, .replenish-card:hover { border-color: hsl(var(--accent) / 0.2); }

                /* Metric Hero */
                .balance-display { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: var(--s-8) 0; }
                .credit-count { 
                    font-family: var(--font-heading); 
                    font-size: 7rem; 
                    font-weight: 950; 
                    line-height: .8; 
                    color: hsl(var(--text));
                    letter-spacing: -0.06em;
                    text-shadow: 0 0 30px hsl(var(--accent) / 0.1);
                }
                .credit-symbol { font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim)); letter-spacing: 0.4em; text-transform: uppercase; margin-top: var(--s-4); opacity: 0.4; }
                
                .meta-stats { display: flex; gap: var(--s-12); margin-top: var(--s-12); border-top: 1px solid var(--glass-border); padding-top: var(--s-8); width: 100%; justify-content: center; }
                .stat-box { display: flex; flex-direction: column; gap: 4px; text-align: center; }
                .stat-label { font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.12em; }
                .stat-value { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-muted)); font-family: var(--font-mono); }
                .stat-value.highlight { color: hsl(var(--success)); }

                /* Pack Styling - Back to preferred look */
                .pack-list { display: flex; flex-direction: column; gap: var(--s-3); flex: 1; }
                .pack-item { 
                    padding: var(--s-4) var(--s-6); 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-md); 
                    cursor: pointer; 
                    display: flex; 
                    justify-content: space-between; 
                    align-items: center; 
                    transition: all 0.2s;
                }
                .pack-item:hover { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--surface-raised) / 0.4); }
                .pack-item.active { border-color: hsl(var(--accent)); background: hsl(var(--accent) / 0.05); }
                
                .pack-info { display: flex; flex-direction: column; gap: 2px; }
                .pack-name { font-size: 0.8125rem; font-weight: 750; color: hsl(var(--text)); }
                .pack-credits { font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.05em; }
                .pack-price { font-family: var(--font-mono); font-size: 1.125rem; font-weight: 900; color: hsl(var(--accent)); }

                .card-actions-row { margin-top: var(--s-8); }
                .btn-block { width: 100%; border-radius: var(--radius-md); font-weight: 850; letter-spacing: 0.05em; }

                /* Telemetry Section Header */
                .history-section { margin-top: var(--s-20); }
                .history-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: var(--s-8); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-6); }
                
                .history-title h2 { font-size: 1.25rem; font-weight: 900; letter-spacing: -0.02em; }
                .history-title p { font-size: 0.75rem; margin-top: 4px; }

                .telemetry-badge { 
                    display: flex; 
                    align-items: baseline; 
                    gap: var(--s-3); 
                    padding: var(--s-2) var(--s-4); 
                    background: hsl(var(--surface-raised)); 
                    border: 1px solid var(--glass-border); 
                    border-radius: 4px;
                }
                .badge-label { font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.6; }
                .badge-value { font-family: var(--font-mono); font-size: 0.8125rem; font-weight: 800; color: hsl(var(--accent)); }

                /* Table Interior - Increased spacing & padding */
                .usage-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); overflow: hidden; }
                .usage-table { width: 100%; border-collapse: collapse; text-align: left; }
                .usage-table th { padding: var(--s-6) var(--s-10); font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.2em; border-bottom: 1px solid var(--glass-border); background: hsl(var(--surface-raised) / 0.5); }
                .usage-table td { padding: var(--s-6) var(--s-10); font-size: 0.75rem; border-bottom: 1px solid var(--glass-border); color: hsl(var(--text-muted)); font-family: var(--font-mono); line-height: 1.6; transition: all 0.2s; }
                .usage-table tr:last-child td { border-bottom: none; }
                .usage-table tr:hover td { background: hsl(var(--surface-raised) / 0.3); color: hsl(var(--text)); }
                
                .id-cell { opacity: 0.8; }
                .latency-cell { color: hsl(var(--success) / 0.82); font-weight: 800; }
                
                .cell-link { color: hsl(var(--accent)); text-decoration: none; font-weight: 800; transition: opacity 0.2s; }
                .cell-link:hover { opacity: 0.8; }
                
                .status-chip { font-size: 0.55rem; font-weight: 900; padding: 2px 10px; border-radius: 4px; border: 1px solid currentColor; text-transform: uppercase; letter-spacing: 0.1em; }
                .status-chip.completed { color: hsl(var(--success)); background: hsl(var(--success) / 0.05); border-color: hsl(var(--success) / 0.2); }
                .status-chip.failed { color: hsl(var(--error)); background: hsl(var(--error) / 0.05); border-color: hsl(var(--error) / 0.2); }
                

                @media (max-width: 1050px) {
                    .credits-layout { grid-template-columns: 1fr; }
                    .credit-count { font-size: 5rem; }
                    .history-header { flex-direction: column; align-items: flex-start; gap: var(--s-4); }
                }
                "
            </style>
        </div>
    }
}

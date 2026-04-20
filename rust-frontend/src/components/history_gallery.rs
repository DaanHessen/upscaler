use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, RefreshCw, AlertCircle, Zap};
use crate::auth::use_auth;
use crate::api::{ApiClient, HistoryItem};

#[component]
pub fn HistoryGallery() -> impl IntoView {
    let auth = use_auth();
    let history = LocalResource::new(
        move || { 
            let session = auth.session.get();
            async move {
                if let Some(s) = session {
                    ApiClient::get_history(Some(&s.access_token)).await
                } else {
                    // During hydration, we wait for the session effect to populate auth.session
                    std::future::pending::<Result<Vec<HistoryItem>, String>>().await
                }
            }
        }
    );

    view! {
        <div class="history-container fade-in">
            <div class="history-header">
                <div class="header-main">
                    <h1>"UPSYL Vault"</h1>
                    <p class="muted">"Secure vault of previously reconstructed assets."</p>
                </div>
                <button class="btn btn-secondary btn-sm" on:click=move |_| history.refetch()>
                    <RefreshCw size={14} />
                    "Refresh Vault"
                </button>
            </div>

            <Suspense fallback=move || view! { 
                <div class="loading-grid">
                    {(0..6).map(|_| view! { <div class="skeleton-card"></div> }).collect_view()}
                </div> 
            }>
                {move || Suspend::new(async move {
                    match history.await {
                        Ok(items) => {
                            let filtered_items: Vec<_> = items.into_iter().filter(|item| item.status != "EXPIRED").collect();
                            if filtered_items.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <ImageIcon size={48} />
                                        <h3>"Empty Vault"</h3>
                                        <p>"Successfully processed images will appear here for 24 hours."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="history-grid">
                                        {filtered_items.into_iter().map(|item| view! { <HistoryCard item=item /> }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(msg) => {
                            view! {
                                <div class="error-panel">
                                    <AlertCircle size={24} />
                                    <p>{msg}</p>
                                    <button class="btn btn-secondary btn-sm" on:click=move |_| history.refetch()>"RETRY"</button>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>

            <style>
                ".history-header { margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); display: flex; justify-content: space-between; align-items: flex-end; }
                .vault-subtitle { font-size: 0.875rem; color: hsl(var(--text-dim)); font-weight: 500; }
                
                .history-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
                    gap: var(--s-8);
                }
                
                .empty-state {
                    grid-column: 1 / -1;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    padding: 8rem 2rem;
                    text-align: center;
                    background: hsl(var(--surface));
                    border: 1px solid var(--glass-border);
                    border-radius: var(--radius-lg);
                    color: hsl(var(--text-dim));
                }
                .empty-state h3 { font-family: var(--font-heading); color: hsl(var(--text)); margin-top: var(--s-4); font-size: 1.25rem; font-weight: 800; letter-spacing: -0.02em; }
                .empty-state p { font-size: 0.875rem; max-width: 320px; margin-top: var(--s-2); opacity: 0.8; }

                .loading-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--s-8); }
                .skeleton-card { height: 320px; background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); position: relative; overflow: hidden; }
                .skeleton-card::after { content: ''; position: absolute; inset: 0; background: linear-gradient(90deg, transparent, hsl(var(--text) / 0.03), transparent); animation: shimmer 1.5s infinite; }
                @keyframes shimmer { 0% { transform: translateX(-100%); } 100% { transform: translateX(100%); } }

                @media (max-width: 900px) {
                    .history-header { margin-bottom: var(--s-10); }
                    .header-main h1 { font-size: 1.75rem; }
                    .history-grid, .loading-grid { grid-template-columns: 1fr; }
                }
                "
            </style>
        </div>
    }
}

#[component]
fn HistoryCard(item: HistoryItem) -> impl IntoView {
    let status_label = match item.status.as_str() {
        "COMPLETED" => "VERIFIED",
        "FAILED" => "FAILED",
        "PROCESSING" => "ACTIVE",
        "PENDING" => "QUEUED",
        _ => "UNKNOWN",
    };
    
    let status_class = match item.status.as_str() {
        "COMPLETED" => "status-pill success",
        "FAILED" => "status-pill error",
        _ => "status-pill active",
    };

    view! {
        <div class="card history-card">
            <div class="card-visual">
                {
                    match item.image_url.clone() {
                        Some(url) => view! { <img src=url /> }.into_any(),
                        _ => view! { <div class="visual-placeholder"><ImageIcon size={32} /></div> }.into_any(),
                    }
                }
                <div class="badge-overlay">
                    <span class="quality-badge">{item.quality}</span>
                </div>
            </div>
            
            <div class="card-details">
                <div class="details-top">
                    <div class=status_class>{status_label}</div>
                    <div class="meta-date">
                        <Calendar size={12} />
                        <span>{item.created_at}</span>
                    </div>
                </div>
                
                <div class="details-main">
                    <div class="style-tag">
                        <Zap size={10} />
                        <span>{item.style.unwrap_or_else(|| "AUTO".to_string())}</span>
                    </div>
                    
                    <div class="card-actions">
                        {
                            match item.image_url.clone() {
                                Some(url) => view! {
                                    <a href=url target="_blank" class="btn btn-primary btn-sm" style="flex: 1; text-decoration: none;">
                                        <Download size={12} />
                                        "EXPORT"
                                    </a>
                                }.into_any(),
                                _ => view! {
                                    <button class="btn btn-secondary btn-sm" disabled=true style="flex: 1; opacity: 0.5;">
                                        "UNAVAILABLE"
                                    </button>
                                }.into_any(),
                            }
                        }
                    </div>
                </div>
            </div>

            <style>
                ".history-card { display: flex; flex-direction: column; background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); overflow: hidden; transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
                .history-card:hover { border-color: hsl(var(--accent) / 0.4); transform: translateY(-4px); box-shadow: var(--shadow-xl); }

                .card-visual { height: 220px; background: hsl(var(--surface-raised)); position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden; border-bottom: 1px solid var(--glass-border); }
                .card-visual img { width: 100%; height: 100%; object-fit: cover; transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1); }
                .history-card:hover .card-visual img { transform: scale(1.05); }
                .visual-placeholder { color: hsl(var(--border) / 0.5); }
                
                .badge-overlay { position: absolute; top: var(--s-4); right: var(--s-4); }
                .quality-badge { font-size: 0.625rem; font-weight: 900; background: rgba(0,0,0,0.6); color: white; padding: 0.25rem 0.5rem; border-radius: 4px; border: 1px solid rgba(255,255,255,0.1); font-family: var(--font-mono); backdrop-filter: blur(8px); letter-spacing: 0.05em; }

                .card-details { padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-4); flex: 1; }
                .details-top { display: flex; justify-content: space-between; align-items: center; }
                
                .status-pill { font-size: 0.6rem; font-weight: 900; padding: 0.2rem 0.5rem; border-radius: 4px; border: 1px solid currentColor; letter-spacing: 0.1em; text-transform: uppercase; }
                .status-pill.success { color: hsl(var(--success)); background: hsl(var(--success) / 0.1); border-color: hsl(var(--success) / 0.2); }
                .status-pill.error { color: hsl(var(--error)); background: hsl(var(--error) / 0.1); border-color: hsl(var(--error) / 0.2); }
                .status-pill.active { color: hsl(var(--accent)); background: hsl(var(--accent) / 0.1); border-color: hsl(var(--accent) / 0.2); }

                .meta-date { display: flex; align-items: center; gap: var(--s-2); font-size: 0.6875rem; color: hsl(var(--text-dim)); font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; }
                
                .details-main { display: flex; align-items: center; justify-content: space-between; margin-top: auto; padding-top: var(--s-4); border-top: 1px solid var(--glass-border); gap: var(--s-4); }
                .style-tag { display: flex; align-items: center; gap: 0.4rem; font-size: 0.625rem; color: hsl(var(--text-muted)); font-weight: 800; text-transform: uppercase; letter-spacing: 0.1em; }

                .card-actions { display: flex; gap: 0.5rem; flex: 1; }
                "
            </style>
        </div>
    }
}

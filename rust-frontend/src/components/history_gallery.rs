use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, RefreshCw, AlertCircle, Zap};
use crate::auth::use_auth;
use crate::api::{ApiClient, HistoryItem};

#[component]
pub fn HistoryGallery() -> impl IntoView {
    let auth = use_auth();
    let history = LocalResource::new(
        move || { 
            let token = auth.session.get().map(|s| s.access_token);
            async move {
                ApiClient::get_history(token.as_deref()).await
            }
        }
    );

    view! {
        <div class="history-container fade-in">
            <div class="history-header">
                <div class="header-main">
                    <h1>"Upscaling History"</h1>
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
                ".history-container { width: 100%; max-width: 900px; margin: 0 auto; }
                .history-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 3.5rem; border-bottom: 1px solid var(--border-color); padding-bottom: 2rem; }
                .header-main h1 { font-size: 2.25rem; font-weight: 800; letter-spacing: -0.04em; margin-bottom: 0.25rem; }
                .header-main p { font-size: 0.9rem; }

                .history-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 1.5rem; }
                
                .loading-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 1.5rem; }
                .skeleton-card { height: 280px; background: var(--surface-color); border: 1px solid var(--border-color); border-radius: 8px; position: relative; overflow: hidden; }
                .skeleton-card::after { content: ''; position: absolute; inset: 0; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.03), transparent); animation: shimmer 1.5s infinite; }
                @keyframes shimmer { 0% { transform: translateX(-100%); } 100% { transform: translateX(100%); } }

                @media (max-width: 900px) {
                    .history-header { margin-bottom: 2.5rem; }
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
                        <Zap size={12} />
                        <span>{item.style.unwrap_or_else(|| "AUTO".to_string())}</span>
                    </div>
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

            <style>
                ".history-card { display: flex; flex-direction: column; transition: transform 0.2s ease, border-color 0.2s ease; }
                .history-card:hover { border-color: var(--accent); transform: translateY(-2px); }

                .card-visual { height: 180px; background: #000; position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden; border-bottom: 1px solid var(--border-color); }
                .card-visual img { width: 100%; height: 100%; object-fit: cover; }
                .visual-placeholder { color: var(--border-color); }
                
                .badge-overlay { position: absolute; bottom: 0.75rem; right: 0.75rem; }
                .quality-badge { font-size: 0.6rem; font-weight: 800; background: rgba(0,0,0,0.8); color: #fff; padding: 0.2rem 0.4rem; border-radius: 4px; border: 1px solid rgba(255,255,255,0.15); font-family: var(--font-mono); }

                .card-details { padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; flex: 1; }
                .details-top { display: flex; justify-content: space-between; align-items: center; }
                
                .status-pill { font-size: 0.6rem; font-weight: 800; padding: 0.2rem 0.5rem; border-radius: 4px; border: 1px solid currentColor; letter-spacing: 0.05em; }
                .status-pill.success { color: var(--success); background: rgba(63, 185, 80, 0.1); }
                .status-pill.error { color: var(--error); background: rgba(248, 81, 73, 0.1); }
                .status-pill.active { color: var(--accent); background: rgba(88, 166, 255, 0.1); }
                .status-pill.muted { color: var(--text-muted); background: var(--surface-lighter); }

                .meta-date { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; color: var(--text-muted); font-weight: 500; }
                
                .details-main { display: flex; gap: 0.5rem; }
                .style-tag { display: flex; align-items: center; gap: 0.4rem; font-size: 0.7rem; color: var(--text-color); font-weight: 600; background: var(--surface-lighter); padding: 0.25rem 0.5rem; border-radius: 4px; text-transform: uppercase; }

                .card-actions { margin-top: auto; display: flex; gap: 0.5rem; }
                "
            </style>
        </div>
    }
}

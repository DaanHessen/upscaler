use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, RefreshCw, Zap};
use crate::auth::use_auth;
use crate::api::HistoryItem;
use leptos_router::components::A;

#[component]
pub fn HistoryGallery() -> impl IntoView {
    let auth = use_auth();
    // Trigger throttled telemetry sync on mount
    Effect::new(move |_| {
        auth.sync_telemetry(false);
    });

    let _history = auth.history;

    view! {
        <div class="history-container fade-in">
            <div class="history-header">
                <div class="header-main">
                    <h1 class="text-gradient stagger-1">"History"</h1>
                    <p class="muted stagger-2">"Your previously upscaled images."</p>
                </div>
                <button class="btn btn-secondary btn-sm stagger-3" on:click=move |_| auth.sync_telemetry(true)>
                    <RefreshCw size={14} />
                    "REFRESH"
                </button>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="history-grid">
                        {(0..8).map(|_| view! { 
                            <div class="card history-card skeleton" style="height: 380px; opacity: 0.5;"></div> 
                        }).collect_view()}
                    </div>
                }
            }>
                {move || {
                    let h = auth.history.get();
                    match h {
                        Some(items) => {
                            let filtered_items: Vec<_> = items.into_iter().filter(|item| item.status != "EXPIRED").collect();
                            if filtered_items.is_empty() {
                                view! {
                                    <div class="empty-state stagger-2">
                                        <div class="empty-icon"><ImageIcon size={48} /></div>
                                        <h3>"Gallery Empty"</h3>
                                        <p>"Successfully processed images will appear here. Records are preserved for 24 hours to ensure privacy and storage efficiency."</p>
                                        <A href="/" attr:class="btn btn-secondary" attr:style="margin-top: var(--s-8)">"Back to Studio"</A>
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
                        None => view! {
                            <div class="history-grid">
                                {(0..8).map(|_| view! { 
                                    <div class="card history-card skeleton" style="height: 380px; opacity: 0.5;"></div> 
                                }).collect_view()}
                            </div> 
                        }.into_any()
                    }
                }}
            </Suspense>

            <style>
                ".history-container { width: 100%; max-width: 1200px; margin: 0 auto; min-height: 60vh; }
                .history-header { margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); display: flex; justify-content: space-between; align-items: flex-end; }
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
                    padding: 10rem 2rem;
                    text-align: center;
                    background: transparent;
                    color: hsl(var(--text-dim));
                }
                .empty-icon { opacity: 0.2; transform: scale(1.2); filter: drop-shadow(0 0 20px hsl(var(--text) / 0.1)); }
                .empty-state h3 { font-family: var(--font-heading); color: hsl(var(--text)); margin-top: var(--s-8); font-size: 1.5rem; font-weight: 800; letter-spacing: -0.04em; }
                .empty-state p { font-size: 0.9375rem; max-width: 400px; margin-top: var(--s-3); opacity: 0.6; line-height: 1.6; }

                .loading-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--s-8); }
                .skeleton-card { height: 320px; background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); position: relative; overflow: hidden; }
                
                @media (max-width: 900px) {
                    .history-header { margin-bottom: var(--s-10); flex-direction: column; align-items: flex-start; gap: var(--s-6); }
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
                    let url = item.preview_url.clone().or_else(|| item.image_url.clone());
                    match url {
                        Some(u) => {
                            let (loaded, set_loaded) = signal(false);
                            view! { 
                                <img 
                                    src=u 
                                    loading="lazy" 
                                    class:loaded=loaded 
                                    on:load=move |_| set_loaded.set(true)
                                /> 
                            }.into_any()
                        },
                        _ => view! { <div class="visual-placeholder"><ImageIcon size={32} /></div> }.into_any(),
                    }
                }
                <div class="badge-overlay">
                    <span class="quality-badge">{item.quality.replace(" RECON", "")}</span>
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
                            let status = item.status.clone();
                            match item.image_url.clone() {
                                Some(url) => view! {
                                    <a href=url target="_blank" class="btn btn-primary btn-sm" style="flex: 1; text-decoration: none;">
                                        <Download size={12} />
                                        "DOWNLOAD"
                                    </a>
                                }.into_any(),
                                _ if status == "FAILED" => view! {
                                    <button class="btn btn-secondary btn-sm" disabled=true style="flex: 1; opacity: 0.35; filter: grayscale(1); cursor: not-allowed; border-color: var(--glass-border); gap: var(--s-1);">
                                        <Download size={12} />
                                        "DOWNLOAD"
                                    </button>
                                }.into_any(),
                                _ => view! {
                                    <button class="btn btn-secondary btn-sm" disabled=true style="flex: 1; opacity: 0.4; filter: grayscale(1); cursor: not-allowed; border-color: var(--glass-border);">
                                        <div style="display: flex; align-items: center; gap: 0.4rem; opacity: 0.5;"><Download size={14} /> "LOCKED"</div>
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

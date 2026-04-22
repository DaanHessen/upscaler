use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, RefreshCw, Zap};
use crate::auth::use_auth;
use crate::api::HistoryItem;
use leptos_router::components::A;

#[component]
pub fn HistoryGallery() -> impl IntoView {
    let auth = use_auth();
    // Trigger throttled telemetry sync on mount (handles SPA navigation)
    Effect::new(move |_| {
        auth.sync_telemetry(false);
    });

    let _history = auth.history;

    view! {
        <div class="history-container fade-in">
            <div class="history-header">
                <div class="header-main">
                    <h1 class="hero-title text-gradient stagger-1">"History"</h1>
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

        </div>
    }
}

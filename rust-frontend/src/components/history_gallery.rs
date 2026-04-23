use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, Zap};
use crate::auth::use_auth;
use crate::api::HistoryItem;
use leptos_router::components::A;

#[component]
pub fn HistoryGallery() -> impl IntoView {
    let auth = use_auth();
    // Trigger proactive telemetry sync on mount
    Effect::new(move |_| {
        let force = auth.history.get().is_none();
        auth.sync_telemetry(force);
    });

    let _history = auth.history;

    view! {
        <div class="history-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"History"</h1>
                    <p class="muted stagger-2">"Your previously upscaled images."</p>
                </div>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="history-grid">
                        {(0..8).map(|_| view! { 
                            <div class="card history-card-v2 skeleton" style="height: 420px; opacity: 0.5;"></div> 
                        }).collect_view()}
                    </div>
                }
            }>
                {move || {
                    let h = auth.history.get();
                    match h {
                        Some(items) => {
                            let filtered_items: Vec<_> = items.into_iter().filter(|item| item.status != "EXPIRED" && item.quality != "TOP-UP").collect();
                            if filtered_items.is_empty() {
                                view! {
                                    <div class="empty-state stagger-2">
                                        <div class="empty-icon"><ImageIcon size={48} /></div>
                                        <h3>"Gallery Empty"</h3>
                                        <p>"Successfully processed images will appear here. Records are preserved for 24 hours to ensure privacy and storage efficiency."</p>
                                        <A href="/" attr:class="btn btn-secondary" attr:style="margin-top: var(--s-8)">"BACK TO STUDIO"</A>
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
                                    <div class="card history-card-v2 skeleton" style="height: 420px; opacity: 0.5;"></div> 
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
        "COMPLETED" => "",
        "FAILED" => "FAILED",
        "PROCESSING" => "ACTIVE",
        "PENDING" => "QUEUED",
        _ => "UNKNOWN",
    };
    
    let status_class = match item.status.as_str() {
        "COMPLETED" => "success",
        "FAILED" => "error",
        _ => "active",
    };

    let status_icon = match item.status.as_str() {
        "COMPLETED" => view! { <Zap size={10} /> }.into_any(),
        _ => view! { <crate::components::icons::RefreshCw size={10} /> }.into_any(),
    };

    view! {
        <div class="card history-card-v2">
            <div class="card-visual-v2">
                {
                    let url = item.preview_url.clone().or_else(|| item.image_url.clone());
                    match url {
                        Some(u) => {
                            let (loaded, set_loaded) = signal(false);
                            view! { 
                                <img 
                                    src=u 
                                    loading="lazy" 
                                    decoding="async"
                                    class:loaded=loaded 
                                    on:load=move |_| set_loaded.set(true)
                                    alt="Upscaled result"
                                /> 
                            }.into_any()
                        },
                        _ => view! { <div class="visual-placeholder"><ImageIcon size={32} /></div> }.into_any(),
                    }
                }
                <div class="visual-overlay"></div>
                <div class="badge-overlay-v2">
                    <span class="quality-badge-v2">{item.quality.replace(" RECON", "")}</span>
                </div>
            </div>
            
            <div class="card-content-v2">
                <div class="card-header-v2">
                    <div class=format!("card-tag-v2 status-{}", status_class)>
                        {status_icon}
                        <span>{status_label}</span>
                    </div>
                    {move || if item.latency_ms > 0 {
                        view! {
                            <div class="latency-v2">
                                {format!("{:.1}s", item.latency_ms as f32 / 1000.0)}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }}
                </div>

                <div class="card-meta-v2">
                    <div class="date-row-v2">
                        <Calendar size={12} />
                        <span>{item.created_at[..10].to_string()}</span>
                    </div>
                </div>
                
                <div class="card-footer-v2">
                    {
                        let status = item.status.clone();
                        match item.image_url.clone() {
                            Some(url) => view! {
                                <a href=url target="_blank" class="btn btn-primary btn-sm studio-download-btn" style="width: 100%; text-decoration: none;">
                                    <Download size={14} />
                                    "DOWNLOAD ASSET"
                                </a>
                            }.into_any(),
                            _ if status == "FAILED" => view! {
                                <button class="btn btn-secondary btn-sm" disabled=true style="width: 100%; opacity: 0.35; filter: grayscale(1); cursor: not-allowed; border-color: var(--glass-border); gap: var(--s-1);">
                                    <Download size={14} />
                                    "FAILED"
                                </button>
                            }.into_any(),
                            _ => view! {
                                <button class="btn btn-secondary btn-sm" disabled=true style="width: 100%; opacity: 0.4; filter: grayscale(1); cursor: not-allowed; border-color: var(--glass-border);">
                                    <div style="display: flex; align-items: center; gap: 0.4rem; opacity: 0.5;"><Download size={14} /> "LOCKED"</div>
                                </button>
                            }.into_any(),
                        }
                    }
                </div>
            </div>
        </div>
    }
}

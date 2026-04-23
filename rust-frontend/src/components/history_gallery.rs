use leptos::prelude::*;
use crate::components::icons::{ImageIcon, Download, Calendar, Zap};
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

            <style>
                ".history-grid { 
                    display: grid; 
                    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); 
                    gap: var(--s-8); 
                    margin-top: var(--s-8); 
                }

                .history-card-v2 {
                    background: hsl(var(--surface-bright));
                    border: 1px solid rgba(255,255,255,0.03);
                    border-radius: var(--radius-lg);
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                    transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
                }

                .history-card-v2:hover {
                    transform: translateY(-4px);
                    border-color: hsl(var(--accent) / 0.3);
                    box-shadow: 0 20px 40px rgba(0,0,0,0.4), 0 0 0 1px hsl(var(--accent) / 0.1);
                }

                .card-visual-v2 {
                    height: 280px;
                    background: #000;
                    position: relative;
                    overflow: hidden;
                }

                .card-visual-v2 img {
                    width: 100%;
                    height: 100%;
                    object-fit: cover;
                    opacity: 0;
                    transition: opacity 0.6s cubic-bezier(0.16, 1, 0.3, 1);
                }

                .card-visual-v2 img.loaded {
                    opacity: 1;
                }

                .visual-overlay {
                    position: absolute;
                    inset: 0;
                    background: linear-gradient(to top, rgba(0,0,0,0.4) 0%, transparent 40%);
                    pointer-events: none;
                }

                .badge-overlay-v2 {
                    position: absolute;
                    top: 12px;
                    right: 12px;
                    z-index: 10;
                }

                .quality-badge-v2 {
                    background: rgba(0,0,0,0.6);
                    backdrop-filter: blur(8px);
                    padding: 4px 10px;
                    border-radius: 6px;
                    color: white;
                    font-size: 0.625rem;
                    font-weight: 900;
                    letter-spacing: 0.1em;
                    border: 1px solid rgba(255,255,255,0.1);
                }

                .card-content-v2 {
                    padding: var(--s-6);
                    display: flex;
                    flex-direction: column;
                    gap: var(--s-6);
                    flex: 1;
                }

                .card-header-v2 {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }

                .card-tag-v2 {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    padding: 4px 12px;
                    border-radius: 100px;
                    font-size: 0.625rem;
                    font-weight: 900;
                    letter-spacing: 0.1em;
                    text-transform: uppercase;
                    border: 1px solid transparent;
                }

                .card-tag-v2.status-success {
                    background: hsl(var(--success) / 0.1);
                    color: hsl(var(--success));
                    border-color: hsl(var(--success) / 0.1);
                }

                .card-tag-v2.status-error {
                    background: hsl(var(--error) / 0.1);
                    color: hsl(var(--error));
                    border-color: hsl(var(--error) / 0.1);
                }

                .card-tag-v2.status-active {
                    background: hsl(var(--accent) / 0.1);
                    color: hsl(var(--accent));
                    border-color: hsl(var(--accent) / 0.1);
                }

                .latency-v2 {
                    font-size: 0.6875rem;
                    font-weight: 800;
                    color: hsl(var(--success));
                    font-family: var(--font-mono);
                }

                .card-meta-v2 {
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                }

                .style-row-v2 {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                }

                .meta-label-v2 {
                    font-size: 0.625rem;
                    font-weight: 850;
                    color: hsl(var(--text-dim) / 0.4);
                    letter-spacing: 0.1em;
                }

                .meta-val-v2 {
                    font-size: 0.625rem;
                    font-weight: 900;
                    color: hsl(var(--text-dim));
                    letter-spacing: 0.05em;
                }

                .date-row-v2 {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    font-size: 0.6875rem;
                    color: hsl(var(--text-dim) / 0.6);
                    font-weight: 600;
                }

                .card-footer-v2 {
                    margin-top: auto;
                }

                .studio-download-btn {
                    height: 44px;
                    font-weight: 850;
                    letter-spacing: 0.1em;
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
                    <div class="style-row-v2">
                        <span class="meta-label-v2">"ENGINE:"</span>
                        <span class="meta-val-v2">{item.style.unwrap_or_else(|| "AUTO".to_string())}</span>
                    </div>
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

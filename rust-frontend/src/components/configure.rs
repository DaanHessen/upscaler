use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle, Download, Info, Maximize};
use wasm_bindgen::JsCast;

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();

    let (is_submitting, set_is_submitting) = signal(false);
    let (processing_job, set_processing_job) = signal(Option::<uuid::Uuid>::None);
    let (engine_status, set_engine_status) = signal(Option::<PollResponse>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_dragging, set_is_dragging) = signal(false);

    let (before_url, set_before_url) = signal(Option::<String>::None);
    let (view_mode, set_view_mode) = signal("compare".to_string());
    let (zoom_level, set_zoom_level) = signal(1.0f64);

    Effect::new(move |_| {
        if let Some(file) = global_state.temp_file.get() {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                set_before_url.set(Some(url.clone()));
                on_cleanup(move || {
                    let _ = web_sys::Url::revoke_object_url(&url);
                });
            }
        } else {
            set_before_url.set(None);
        }
    });

    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let c = cancelled.clone();
    on_cleanup(move || c.store(true, std::sync::atomic::Ordering::Relaxed));

    let c_effect = cancelled.clone();
    Effect::new(move |_| {
        if let Some(job_id) = processing_job.get() {
            let token = auth.session.get().map(|s| s.access_token);
            let state = global_state;
            let c_loop = c_effect.clone();
            leptos::task::spawn_local(async move {
                loop {
                    if c_loop.load(std::sync::atomic::Ordering::Relaxed) { break; }
                    match ApiClient::poll_job(job_id, token.as_deref()).await {
                        Ok(resp) => {
                            let r: PollResponse = resp.clone();
                            set_engine_status.set(Some(r.clone()));
                            if r.status == "COMPLETED" {
                                if let Some(url) = r.image_url {
                                    state.set_preview_base64.set(Some(url));
                                }
                                // Return to editor state but keep the preview
                                set_processing_job.set(None);
                                break;
                            }
                            if r.status == "FAILED" {
                                set_error_msg.set(Some(r.error.unwrap_or_else(|| "Upscale failed.".to_string())));
                                set_processing_job.set(None);
                                break;
                            }
                        },
                        Err(_) => break,
                    }
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }
            });
        }
    });

    let on_file_input = move |ev: leptos::web_sys::Event| {
        let input: web_sys::HtmlInputElement = leptos::prelude::event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                if file.size() > 20_000_000.0 {
                    set_error_msg.set(Some("File exceeds 20MB limit.".to_string()));
                    return;
                }
                if !file.type_().starts_with("image/") {
                    set_error_msg.set(Some("Invalid file type. Only images are allowed.".to_string()));
                    return;
                }
                set_error_msg.set(None);
                global_state.set_temp_file.set(Some(file));
                global_state.set_preview_base64.set(None);
            }
        }
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
        if let Some(data) = ev.data_transfer() {
            if let Some(files) = data.files() {
                if let Some(file) = files.get(0) {
                    if file.size() > 20_000_000.0 {
                        set_error_msg.set(Some("File exceeds 20MB limit.".to_string()));
                        return;
                    }
                    if !file.type_().starts_with("image/") {
                        set_error_msg.set(Some("Invalid file type. Only images are allowed.".to_string()));
                        return;
                    }
                    set_error_msg.set(None);
                    global_state.set_temp_file.set(Some(file));
                    global_state.set_preview_base64.set(None);
                }
            }
        }
    };

    let handle_upscale = move |_| {
        if let Some(file) = global_state.temp_file.get() {
            let q_val: String = global_state.quality.get();
            let cost = if q_val == "4K" { 4 } else { 2 };
            if let Some(current) = auth.credits.get() {
                if current < cost {
                    set_error_msg.set(Some("Insufficient credits.".to_string()));
                    return;
                }
            }
            
            set_is_submitting.set(true);
            set_error_msg.set(None);
            let token = auth.session.get().map(|s| s.access_token);
            let s_val: String = global_state.style.get();
            let t_val: f32 = global_state.temperature.get();
            let auth_ctx = auth;
            let p_settings = PromptSettings {
                keep_depth_of_field: global_state.keep_depth_of_field.get(),
                lighting: global_state.lighting.get(),
                thinking_level: global_state.thinking_level.get(),
                seed: global_state.seed.get(),
            };
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, &p_settings, token.as_deref()).await {
                    Ok(resp) => {
                        auth_ctx.set_credits.update(|c| if let Some(cv) = c { *cv -= cost; });
                        auth_ctx.sync_telemetry(true);
                        set_processing_job.set(Some(resp.job_id));
                        set_is_submitting.set(false);
                    },
                    Err(e) => { 
                        set_error_msg.set(Some(format!("Upload failed: {}", e))); 
                        set_is_submitting.set(false);
                    }
                }
            });
        }
    };

    let handle_try_again = move |_| {
        global_state.set_preview_base64.set(None);
        set_engine_status.set(None);
        set_processing_job.set(None);
        set_zoom_level.set(1.0);
    };

    let stage_info = move || {
        if is_submitting.get() {
            return ("INIT", "Preparing", "Securely uploading and analyzing asset...");
        }
        match engine_status.get().map(|s| s.status) {
            Some(s) if s == "PENDING"    => ("QUEUE",  "System Ready",   "Analyzing asset for reconstruction..."),
            Some(s) if s == "PROCESSING" => ("ACTIVE", "Reconstructing", "Gemini Vision is synthesizing high-frequency details."),
            Some(s) if s == "COMPLETED"  => ("DONE",   "Export Ready",   "Enhancement complete. Ready for download."),
            _ =>                            ("IDLE",   "Standby",        "Awaiting engine handshake."),
        }
    };

    let nav_history = navigate.clone();

    view! {
        <div class="editor-shell fade-in">

            // ── Canvas ──────────────────────────────────────────
            <div class="editor-canvas-area"
                 on:dragover=move |ev| { ev.prevent_default(); set_is_dragging.set(true); }
                 on:dragleave=move |_| set_is_dragging.set(false)
                 on:drop=on_drop
            >
                <div class="canvas-grid"></div>

                {move || {
                    let preview = global_state.preview_base64.get();
                    let before = before_url.get();
                    
                    match (before, preview) {
                        (Some(before), Some(after)) => view! {
                            <div class="canvas-view-container animate-in">
                                <div class="slider-fill">
                                    <crate::components::comparison_slider::ComparisonSlider 
                                        images=vec![(before, after.clone())] 
                                        zoom=zoom_level.get()
                                    />
                                </div>

                                // Viewer Controls
                                <div class="viewer-controls">
                                    <div class="viewer-ctrl-group">
                                        <button 
                                            class:active=move || view_mode.get() == "compare"
                                            on:click=move |_| set_view_mode.set("compare".to_string())
                                        >"Compare"</button>
                                        <button 
                                            class:active=move || view_mode.get() == "original"
                                            on:click=move |_| set_view_mode.set("original".to_string())
                                        >"Original"</button>
                                        <button 
                                            class:active=move || view_mode.get() == "upscaled"
                                            on:click=move |_| set_view_mode.set("upscaled".to_string())
                                        >"Upscaled"</button>
                                    </div>
                                    
                                    <div class="viewer-ctrl-divider"></div>

                                    <div class="viewer-ctrl-group">
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z + 0.5).min(4.0))
                                            title="Zoom In"
                                        ><Maximize size={14} /></button>
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z - 0.5).max(1.0))
                                            title="Zoom Out"
                                        ><Target size={14} /></button>
                                    </div>

                                    <div class="viewer-ctrl-divider"></div>

                                    <div class="viewer-ctrl-group">
                                        <a 
                                            href=after.clone() 
                                            target="_blank" 
                                            class="viewer-action-btn"
                                            style="text-decoration:none;"
                                        >
                                            <Download size={14} />
                                            <span>"Download"</span>
                                        </a>
                                        <button 
                                            class="viewer-action-btn secondary"
                                            on:click=handle_try_again
                                        >
                                            <RefreshCw size={14} />
                                            <span>"Try Again"</span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        
                        (Some(before), None) => view! {
                            <div class="asset-wrapper stagger-3">
                                <img src=before class="studio-asset" alt="Original" />
                                <div class="corner-accents">
                                    <div class="corner tl"></div>
                                    <div class="corner tr"></div>
                                    <div class="corner bl"></div>
                                    <div class="corner br"></div>
                                </div>
                            </div>
                        }.into_any(),
                        
                        _ => view! {
                            <div class="drop-zone-outer">
                                <div class="drop-zone-inner" on:click=move |_| {
                                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                        if let Some(el) = doc.get_element_by_id("hidden_file_input") {
                                            let html_el: web_sys::HtmlElement = el.unchecked_into();
                                            html_el.click();
                                        }
                                    }
                                }>
                                    <div class="dz-icon-wrap">
                                        <ImageIcon size={28} />
                                    </div>
                                    <p class="dz-title">"Drop image here"</p>
                                    <p class="dz-sub">"or click to browse your files"</p>
                                    <div class="dz-formats">"PNG · JPG · WEBP · MAX 20 MB"</div>
                                </div>
                                <input type="file" id="hidden_file_input" style="display:none;" on:change=on_file_input />
                            </div>
                        }.into_any()
                    }
                }}

                // Drag overlay
                {move || is_dragging.get().then(|| view! {
                    <div class="drag-overlay fade-in">
                        <Download size={40} />
                        <span>"Drop to import"</span>
                    </div>
                })}
            </div>

            // ── Sidebar ─────────────────────────────────────────
            <aside class="editor-sidebar">
                {move || {
                    let submitting = is_submitting.get();
                    let job = processing_job.get();
                    
                    if submitting || job.is_some() {
                        view! {
                            <div class="sidebar-inner fade-in">
                                <div class="polling-view">
                                    <div class="polling-header">
                                        <div class="p-status-pill">
                                            <div class="status-dot pulse"></div>
                                            <span>{move || stage_info().0}</span>
                                        </div>
                                        <h3 class="p-title">{move || stage_info().1}</h3>
                                        <p class="p-desc">{move || stage_info().2}</p>
                                    </div>

                                    <div class="p-progress-rail">
                                        <div class="p-progress-fill" style:width=move || {
                                            if submitting { return "15%".to_string(); }
                                            match engine_status.get().map(|s| s.status) {
                                                Some(s) if s == "PENDING" => "40%".to_string(),
                                                Some(s) if s == "PROCESSING" => "75%".to_string(),
                                                Some(s) if s == "COMPLETED" => "100%".to_string(),
                                                _ => "10%".to_string()
                                            }
                                        }></div>
                                    </div>
                                    
                                    <div class="p-telemetry-grid">
                                        <div class="p-tele-item">
                                            <span class="p-tele-label">"LATENCY"</span>
                                            <span class="p-tele-val">{move || engine_status.get().and_then(|s| s.latency_ms).map(|l| format!("{}ms", l)).unwrap_or_else(|| "---".to_string())}</span>
                                        </div>
                                        <div class="p-tele_item">
                                            <span class="p-tele-label">"TOKEN"</span>
                                            <span class="p_tele-val">{move || job.map(|id| id.to_string().chars().take(8).collect::<String>()).unwrap_or_else(|| "---".to_string())}</span>
                                        </div>
                                    </div>

                                    <div class="p-footer">
                                        <p class="p-hint">"Gemini is analyzing and reconstructing high-frequency details. Please remain on this page."</p>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="sidebar-inner fade-in">
                                <div class="sb-scroll">

                                    // Error
                                    {move || error_msg.get().map(|msg| view! {
                                        <div class="sb-error">
                                            <AlertCircle size={14} />
                                            <span>{msg}</span>
                                        </div>
                                    })}

                                    // ── Resolution ──────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Target size={10} />
                                                <span>"RESOLUTION"</span>
                                            </div>
                                                <div class="res-grid">
                                                    <div
                                                        class=move || if global_state.quality.get() == "2K" { "pack-item active" } else { "pack-item" }
                                                        on:click=move |_| global_state.set_quality.set("2K".to_string())
                                                    >
                                                        <div class="pack-info">
                                                            <span class="res-big-num">"2K"</span>
                                                            <span class="pack-price">"2 credits"</span>
                                                        </div>
                                                    </div>
                                                    <div
                                                        class=move || if global_state.quality.get() == "4K" { "pack-item active" } else { "pack-item" }
                                                        on:click=move |_| global_state.set_quality.set("4K".to_string())
                                                    >
                                                        <div class="pack-info">
                                                            <span class="res-big-num">"4K"</span>
                                                            <span class="pack-price">"4 credits"</span>
                                                        </div>
                                                    </div>
                                                </div>
                                        </div>
                                    </div>

                                    // ── Engine ──────────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Zap size={10} />
                                                <span>"ENGINE"</span>
                                            </div>

                                            // Style
                                            <div class="sb-field">
                                                <label class="sb-label">"Style"</label>
                                                <div class="seg-control">
                                                    <button
                                                        class:active=move || global_state.style.get() == "PHOTOGRAPHY"
                                                        on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                                    >"Photography"</button>
                                                    <button
                                                        class:active=move || global_state.style.get() == "ILLUSTRATION"
                                                        on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                                    >"Illustration"</button>
                                                </div>
                                            </div>

                                            // Creativity
                                            <div class="sb-field" style="margin-top: var(--s-8);">
                                                <div class="sb-label-row" style="margin-bottom: var(--s-3);">
                                                    <label class="sb-label">"Creativity"</label>
                                                    <span class="sb-val-badge">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                                </div>
                                                <div class="slider-wrap">
                                                    <input
                                                        type="range" min="0.0" max="2.0" step="0.1"
                                                        class="studio-slider"
                                                        prop:value=move || global_state.temperature.get().to_string()
                                                        on:input=move |ev| global_state.set_temperature.set(
                                                            leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0)
                                                        )
                                                    />
                                                    <div class="slider-ends">
                                                        <span>"Strict"</span>
                                                        <span>"Creative"</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // ── Advanced ────────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Settings size={10} />
                                                <span>"ADVANCED"</span>
                                            </div>

                                            // Seed
                                            <div class="sb-field">
                                                <div class="sb-label-row" style="margin-bottom: var(--s-3);">
                                                    <label class="sb-label">"Seed"</label>
                                                    <span class="sb-val-badge mono">
                                                        {move || global_state.seed.get()
                                                            .map(|s: u32| format!("#{}", s))
                                                            .unwrap_or_else(|| "AUTO".to_string())}
                                                    </span>
                                                </div>
                                                <div class="seed-row">
                                                    <input
                                                        type="number"
                                                        class="sb-input"
                                                        placeholder="Leave empty for auto"
                                                        prop:value=move || global_state.seed.get()
                                                            .map(|s: u32| s.to_string())
                                                            .unwrap_or_default()
                                                        on:input=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            if val.is_empty() {
                                                                global_state.set_seed.set(None);
                                                            } else if let Ok(s) = val.parse::<u32>() {
                                                                global_state.set_seed.set(Some(s));
                                                            }
                                                        }
                                                    />
                                                    <button
                                                        class="seed-rng-btn"
                                                        title="Randomize seed"
                                                        on:click=move |_| {
                                                            let val = (js_sys::Math::random() * (u32::MAX as f64)) as u32;
                                                            global_state.set_seed.set(Some(val));
                                                        }
                                                    >
                                                        <RefreshCw size={13} />
                                                    </button>
                                                </div>
                                            </div>

                                            // DOF toggle
                                            <div
                                                class=move || if global_state.keep_depth_of_field.get() { "pack-item active dof-row" } else { "pack-item dof-row" }
                                                style="margin-top: var(--s-4); cursor: pointer;"
                                                on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                            >
                                                <div class="pack-info">
                                                    <span class="pack-name">"Depth-of-field lock"</span>
                                                    <span style="font-size: 0.6875rem; color: hsl(var(--text-dim));">"Preserve original focal planes"</span>
                                                </div>
                                                <div class="toggle-track">
                                                    <div class="toggle-thumb"></div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // ── Lighting ────────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Info size={10} />
                                                <span>"LIGHTING"</span>
                                            </div>
                                            <div class="select-wrap">
                                                <select
                                                    class="sb-select"
                                                    on:change=move |ev| global_state.set_lighting.set(
                                                        leptos::prelude::event_target_value(&ev)
                                                    )
                                                    prop:value=move || global_state.lighting.get()
                                                >
                                                    <option value="Original">"Original"</option>
                                                    <option value="Studio">"Studio"</option>
                                                    <option value="Cinematic">"Cinematic"</option>
                                                    <option value="Vivid">"Vivid"</option>
                                                    <option value="Natural">"Natural"</option>
                                                </select>
                                                <div class="select-arrow"></div>
                                            </div>
                                        </div>
                                    </div>

                                </div>

                                // ── CTA ─────────────────────────────
                                <div class="sb-footer">
                                    <button
                                        class="btn btn-primary btn-lg sb-cta"
                                        on:click=handle_upscale
                                        disabled=move || global_state.temp_file.get().is_none()
                                    >
                                        <div class="sb-cta-inner">
                                            <Zap size={16} />
                                            <span>"Initiate Upscale"</span>
                                        </div>
                                        <div class="sb-cta-badge">
                                            {move || if global_state.quality.get() == "4K" { "4" } else { "2" }}
                                            <span style="font-size: 0.625rem; opacity: 0.5; margin-left: 2px;">"CR"</span>
                                        </div>
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </aside>
        </div>

        
    }
}

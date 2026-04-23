use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle, Download, Info, ChevronRight, Maximize};
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

    Effect::new(move |_| {
        if let Some(job_id) = processing_job.get() {
            let token = auth.session.get().map(|s| s.access_token);
            let state = global_state;
            leptos::task::spawn_local(async move {
                loop {
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
                            if r.status == "FAILED" { break; }
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

        <style>
            "
            /* ── Shell ── */
            .editor-shell {
                display: flex;
                height: calc(100vh - 72px);
                overflow: hidden;
            }

            /* ── Canvas ── */
            .editor-canvas-area {
                flex: 1; min-width: 0;
                position: relative;
                display: flex; align-items: center; justify-content: center;
                overflow: hidden;
            }

            .canvas-view-container {
                width: 100%; height: 100%;
                display: flex; flex-direction: column;
                align-items: center; justify-content: center;
                gap: var(--s-6); position: relative;
            }

            .slider-fill {
                width: 100%; height: calc(100% - 100px);
                display: flex; align-items: center; justify-content: center;
            }

            .viewer-controls {
                background: var(--glass);
                backdrop-filter: blur(30px);
                border: 1px solid var(--glass-border);
                border-radius: 100px;
                padding: 10px 14px;
                display: flex; align-items: center; gap: 12px;
                z-index: 100; margin-bottom: var(--s-4);
                box-shadow: 0 10px 40px rgba(0,0,0,0.5);
            }

            .viewer-ctrl-group { display: flex; gap: 4px; align-items: center; }
            .viewer-ctrl-divider { width: 1px; height: 16px; background: var(--glass-border); }

            .viewer-controls button {
                background: transparent; border: none;
                color: hsl(var(--text-dim));
                padding: 6px 14px; font-size: 0.75rem; font-weight: 700;
                border-radius: 100px; cursor: pointer; transition: all 0.2s;
            }
            .viewer-controls button.active {
                background: hsl(var(--accent)); color: white;
                box-shadow: 0 4px 12px hsl(var(--accent) / 0.3);
            }
            .viewer-action-btn {
                background: hsl(var(--accent)); color: white;
                border: none; padding: 6px 14px;
                font-size: 0.75rem; font-weight: 800;
                border-radius: 100px; cursor: pointer;
                display: flex; align-items: center; gap: 6px;
                transition: all 0.2s;
            }
            .viewer-action-btn.secondary { background: rgba(255,255,255,0.05); color: white; }
            .viewer-action-btn:hover { transform: translateY(-1px); filter: brightness(1.1); }

            .canvas-grid {
                position: absolute; inset: 0;
                background-image:
                    linear-gradient(rgba(255,255,255,0.013) 1px, transparent 1px),
                    linear-gradient(90deg, rgba(255,255,255,0.013) 1px, transparent 1px);
                background-size: 44px 44px;
                mask-image: radial-gradient(ellipse 65% 65% at center, black 10%, transparent 100%);
                pointer-events: none;
            }

            /* Asset */
            .asset-wrapper { position: relative; }
            .studio-asset {
                display: block; max-width: 100%; max-height: 62vh;
                border-radius: var(--radius-md);
                box-shadow: 0 32px 80px rgba(0,0,0,0.7), 0 0 0 1px rgba(255,255,255,0.06);
            }
            .corner-accents .corner {
                position: absolute; width: 12px; height: 12px;
                border: 1.5px solid hsl(var(--accent) / 0.45);
            }
            .corner.tl { top:-7px; left:-7px; border-right:0; border-bottom:0; }
            .corner.tr { top:-7px; right:-7px; border-left:0; border-bottom:0; }
            .corner.bl { bottom:-7px; left:-7px; border-right:0; border-top:0; }
            .corner.br { bottom:-7px; right:-7px; border-left:0; border-top:0; }

            /* Drop zone */
            .drop-zone-outer {
                padding: 2px; border-radius: calc(var(--radius-lg) + 2px);
                background: linear-gradient(140deg, hsl(var(--accent) / 0.12) 0%, transparent 45%, hsl(var(--accent) / 0.06) 100%);
                cursor: pointer; transition: background 0.3s;
            }
            .drop-zone-inner {
                background: var(--glass); backdrop-filter: blur(20px);
                border-radius: var(--radius-lg); border: 1px dashed rgba(255,255,255,0.08);
                padding: 3.5rem 4.5rem; display: flex; flex-direction: column;
                align-items: center; gap: var(--s-3);
            }
            .dz-icon-wrap {
                width: 64px; height: 64px; background: hsl(var(--accent) / 0.07);
                border: 1px solid hsl(var(--accent) / 0.13); border-radius: 18px;
                display: flex; align-items: center; justify-content: center;
                color: hsl(var(--accent) / 0.65);
            }
            .dz-title { font-size: 1.125rem; font-weight: 700; color: hsl(var(--text)); }
            .dz-sub { font-size: 0.875rem; color: hsl(var(--text-dim)); }
            .dz-formats { margin-top: var(--s-3); font-size: 0.5625rem; color: hsl(var(--text-dim) / 0.35); text-transform: uppercase; }

            /* Sidebar */
            .editor-sidebar {
                width: 400px; flex-shrink: 0;
                border-left: 1px solid var(--glass-border);
                background: hsl(var(--surface) / 0.7);
                backdrop-filter: blur(24px); display: flex; flex-direction: column;
                overflow: hidden; gap: var(--s-2);
            }
            .sidebar-inner { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
            .sb-scroll { 
                flex: 1; overflow-y: auto; overflow-x: hidden;
                padding: var(--s-6); display: flex; flex-direction: column; gap: var(--s-6); 
            }
            
            /* Custom Scrollbar for Sidebar */
            .sb-scroll::-webkit-scrollbar { width: 4px; }
            .sb-scroll::-webkit-scrollbar-track { background: transparent; }
            .sb-scroll::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.05); border-radius: 100px; }

            /* Polling View */
            .polling-view { padding: var(--s-10) var(--s-8); text-align: center; display: flex; flex-direction: column; gap: var(--s-6); }
            .polling-header { display: flex; flex-direction: column; align-items: center; gap: var(--s-3); }
            .p-status-pill {
                display: flex; align-items: center; gap: 8px;
                background: hsl(var(--accent) / 0.08); border: 1px solid hsl(var(--accent) / 0.15);
                padding: 4px 12px; border-radius: 100px;
                font-size: 0.625rem; font-weight: 900; color: hsl(var(--accent));
            }
            .status-dot { width: 6px; height: 6px; background: hsl(var(--accent)); border-radius: 50%; }
            .status-dot.pulse { animation: status-pulse 2s infinite; }
            @keyframes status-pulse { 0% { transform: scale(1); opacity: 1; } 50% { transform: scale(1.5); opacity: 0.5; } 100% { transform: scale(1); opacity: 1; } }
            
            .p-title { font-size: 1.5rem; font-weight: 800; font-family: var(--font-heading); }
            .p-desc { font-size: 0.875rem; color: hsl(var(--text-dim)); max-width: 240px; margin: 0 auto; }
            
            .p-progress-rail { height: 4px; background: rgba(255,255,255,0.05); border-radius: 4px; overflow: hidden; }
            .p-progress-fill { height: 100%; background: hsl(var(--accent)); transition: width 0.5s ease; width: 0%; }
            
            .p-telemetry-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); }
            .p-tele-item { background: rgba(255,255,255,0.02); padding: var(--s-4); border-radius: var(--radius-md); border: 1px solid var(--glass-border); text-align: left; }
            .p-tele-label { display: block; font-size: 0.5625rem; font-weight: 900; color: hsl(var(--text-dim)); opacity: 0.4; letter-spacing: 0.1em; }
            .p-tele-val { display: block; font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; color: hsl(var(--text)); margin-top: 4px; }
            
            .p-footer { padding-top: var(--s-4); border-top: 1px solid var(--glass-border); }
            .p-hint { font-size: 0.75rem; color: hsl(var(--text-dim)); opacity: 0.6; line-height: 1.5; }

            .editor-card { overflow: visible !important; }
            .editor-card-body { padding: var(--s-8); overflow: visible !important; }
            .res-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); overflow: visible !important; }
            .res-big-num { font-size: 1.5rem; font-weight: 800; }
            .seg-control {
                display: flex;
                background: rgba(255, 255, 255, 0.04);
                border: 1px solid var(--glass-border);
                border-radius: var(--radius-md);
                padding: 4px;
                gap: 4px;
                overflow: visible !important;
            }
            .seg-control button {
                flex: 1;
                background: transparent;
                border: none;
                color: hsl(var(--text-dim));
                padding: 10px 4px;
                font-size: 0.8125rem;
                font-weight: 700;
                border-radius: 6px;
                cursor: pointer;
                transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }
            .seg-control button.active {
                background: hsl(var(--accent));
                color: white;
                box-shadow: 0 4px 12px hsl(var(--accent) / 0.3);
                overflow: visible !important;
            }

            .sb-field { display: flex; flex-direction: column; gap: var(--s-3); overflow: visible !important; }
            .sb-label { font-size: 0.6875rem; font-weight: 850; color: hsl(var(--text-dim) / 0.4); letter-spacing: 0.08em; text-transform: uppercase; }
            .sb-label-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2px; overflow: visible !important; }
            .sb-val-badge { font-family: var(--font-mono); font-size: 0.6875rem; font-weight: 900; color: hsl(var(--accent)); background: hsl(var(--accent) / 0.1); padding: 2px 10px; border-radius: 6px; border: 1px solid hsl(var(--accent) / 0.2); }

            .studio-slider { 
                width: 100%; height: 4px; background: rgba(255,255,255,0.06); border-radius: 4px; appearance: none; outline: none; margin: 8px 0;
            }
            .studio-slider::-webkit-slider-thumb { 
                appearance: none; width: 20px; height: 20px; background: #fff; border-radius: 50%; cursor: pointer; border: 4px solid hsl(var(--accent)); box-shadow: 0 4px 12px rgba(0,0,0,0.6); 
                transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
            }
            .studio-slider::-webkit-slider-thumb:hover { transform: scale(1.15); }
            
            .slider-ends {
                display: flex; justify-content: space-between;
                font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim) / 0.3);
                text-transform: uppercase; letter-spacing: 0.05em; margin-top: -2px;
            }
            
            .sb-footer { 
                padding: var(--s-6); 
                background: linear-gradient(to top, hsl(var(--bg)), transparent);
                border-top: 1px solid var(--glass-border); 
            }
            .sb-cta { 
                width: 100%; height: 56px; display: flex; align-items: center; justify-content: space-between; padding: 0 24px; gap: 12px; 
                background: white; color: black; border: none; font-weight: 900; letter-spacing: 0.05em;
                border-radius: var(--radius-md); transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .sb-cta-inner { display: flex; align-items: center; gap: 12px; }
            .sb-cta-badge { 
                background: rgba(0,0,0,0.08); padding: 4px 12px; border-radius: 8px; font-size: 0.8125rem; font-weight: 900; display: flex; align-items: center; gap: 2px;
            }
            .sb-cta:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 12px 40px rgba(0,0,0,0.5); filter: brightness(1.05); }
            .sb-cta:active:not(:disabled) { transform: translateY(0); }
            .sb-cta:disabled { opacity: 0.3; filter: grayscale(1); cursor: not-allowed; }
            
            .pack-item {
                background: rgba(255,255,255,0.02);
                border: 1px solid var(--glass-border);
                border-radius: var(--radius-lg);
                padding: var(--s-6);
                cursor: pointer;
                transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                display: flex;
                flex-direction: column;
                justify-content: center;
                min-height: 100px;
                overflow: visible !important;
            }
            .pack-info { display: flex; flex-direction: column; gap: 2px; }
            .pack-item:hover { background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.1); }
            .pack-item.active { 
                background: hsl(var(--accent) / 0.08); 
                border-color: hsl(var(--accent));
                box-shadow: 0 0 24px hsl(var(--accent) / 0.2);
            }
            .pack-price { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-dim) / 0.4); margin-top: 4px; }
            .pack-item.active .pack-price { color: hsl(var(--accent)); opacity: 0.8; }
            
            .seed-row { display: flex; gap: var(--s-3); align-items: center; overflow: visible !important; }
            .sb-input { 
                flex: 1; background: rgba(255,255,255,0.04); border: 1px solid var(--glass-border);
                color: white; padding: 12px 14px; border-radius: 8px; font-size: 0.875rem;
                outline: none; transition: all 0.2s;
            }
            .sb-input:focus { border-color: hsl(var(--accent)); }
            
            .seed-rng-btn {
                width: 44px; height: 44px; display: flex; align-items: center; justify-content: center;
                background: rgba(255,255,255,0.05); border: 1px solid var(--glass-border);
                color: hsl(var(--text-dim)); border-radius: 8px; cursor: pointer; transition: all 0.2s;
            }
            .seed-rng-btn:hover { background: rgba(255,255,255,0.1); color: white; }

            .toggle-track { 
                width: 38px; height: 22px; background: rgba(255,255,255,0.1); border-radius: 100px; 
                position: relative; transition: all 0.3s;
            }
            .pack-item.active .toggle-track { background: hsl(var(--accent)); }
            .toggle-thumb { 
                position: absolute; top: 4px; left: 4px; width: 14px; height: 14px; 
                background: white; border-radius: 50%; transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .pack-item.active .toggle-thumb { left: calc(100% - 18px); }
            
            .dof-row { flex-direction: row; align-items: center; justify-content: space-between; min-height: 0; padding: var(--s-6); overflow: visible !important; }

            .select-wrap { position: relative; width: 100%; overflow: visible !important; }
            .sb-select {
                width: 100%; appearance: none; background: rgba(255,255,255,0.04);
                border: 1px solid var(--glass-border); color: white; padding: 12px 14px;
                border-radius: 8px; font-size: 0.875rem; font-weight: 700; cursor: pointer;
                outline: none;
            }
            .sb-select option { background: hsl(var(--surface-raised)); color: white; }
            
            .select-arrow {
                position: absolute; right: 14px; top: 50%; width: 8px; height: 8px;
                border-right: 2px solid hsl(var(--text-dim) / 0.5);
                border-bottom: 2px solid hsl(var(--text-dim) / 0.5);
                transform: translateY(-70%) rotate(45deg);
                pointer-events: none;
            }
            "
        </style>
    }
}

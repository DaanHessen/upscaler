use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle, Download, Info, ChevronRight};
use wasm_bindgen::JsCast;

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();

    let (processing_job, set_processing_job) = signal(Option::<uuid::Uuid>::None);
    let (engine_status, set_engine_status) = signal(Option::<PollResponse>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_dragging, set_is_dragging) = signal(false);

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
                    },
                    Err(e) => { set_error_msg.set(Some(format!("Upload failed: {}", e))); }
                }
            });
        }
    };

    let stage_info = move || {
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

                // Canvas content
                {move || {
                    let img_url = global_state.preview_base64.get();
                    match img_url {
                        Some(url) => view! {
                            <div class="asset-wrapper fade-in">
                                <img src=url class="studio-asset" alt="Upscale Result" />
                                <div class="corner-accents">
                                    <div class="corner tl"></div>
                                    <div class="corner tr"></div>
                                    <div class="corner bl"></div>
                                    <div class="corner br"></div>
                                </div>
                            </div>
                        }.into_any(),
                        None => view! {
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
                    let nav = nav_history.clone();
                    match processing_job.get() {

                        // Settings panel
                        None => view! {
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
                                            <div class="card-tag" style="margin-bottom: var(--s-6);">
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
                                                        <span class="pack-credits">"RESTORE"</span>
                                                    </div>
                                                    <span class="pack-price" style="font-size: 0.8125rem;">"2 cr"</span>
                                                </div>
                                                <div
                                                    class=move || if global_state.quality.get() == "4K" { "pack-item active" } else { "pack-item" }
                                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                                >
                                                    <div class="pack-info">
                                                        <span class="res-big-num">"4K"</span>
                                                        <span class="pack-credits">"ULTRA HD"</span>
                                                    </div>
                                                    <span class="pack-price" style="font-size: 0.8125rem;">"4 cr"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // ── Engine ──────────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-6);">
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
                                            <div class="sb-field" style="margin-top: var(--s-5);">
                                                <div class="sb-label-row">
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
                                            <div class="card-tag" style="margin-bottom: var(--s-6);">
                                                <Settings size={10} />
                                                <span>"ADVANCED"</span>
                                            </div>

                                            // Seed
                                            <div class="sb-field">
                                                <div class="sb-label-row">
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

                                            // DOF toggle  — styled as pack-item
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
                                            <div class="card-tag" style="margin-bottom: var(--s-6);">
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
                                                <ChevronRight size={14} custom_style="transform:rotate(90deg);position:absolute;right:12px;top:50%;margin-top:-7px;pointer-events:none;opacity:0.35;".to_string() />
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
                                        <Zap size={16} />
                                        <span>"Initiate Upscale"</span>
                                        <span class="sb-cta-credit">
                                            {move || if global_state.quality.get() == "4K" { "4 credits" } else { "2 credits" }}
                                        </span>
                                    </button>
                                </div>
                            </div>
                        }.into_any(),

                        // Processing panel
                        Some(_) => {
                            let n = nav.clone();
                            view! {
                                <div class="sidebar-inner processing-panel fade-in">
                                    <div class="proc-body">
                                        <div class="proc-icon">
                                            <Zap size={28} />
                                        </div>
                                        <span class="proc-stage">{move || stage_info().0}</span>
                                        <h3 class="proc-title">{move || stage_info().1}</h3>
                                        <p class="proc-desc">{move || stage_info().2}</p>

                                        <div class="proc-bar-wrap">
                                            <div class="proc-bar-track">
                                                <div class="proc-bar-fill"></div>
                                            </div>
                                            <div class="proc-bar-labels">
                                                <span>"Reconstruction"</span>
                                                <span>"Running"</span>
                                            </div>
                                        </div>

                                        {move || engine_status.get().and_then(|s| s.latency_ms).map(|ms| view! {
                                            <div class="proc-latency fade-in">
                                                <span>"Duration:"</span>
                                                <span class="proc-latency-val">{format!("{:.1}s", ms as f32 / 1000.0)}</span>
                                            </div>
                                        })}
                                    </div>

                                    <div class="proc-footer">
                                        {move || if engine_status.get().map(|s| s.status == "COMPLETED").unwrap_or(false) {
                                            let n2 = n.clone();
                                            view! {
                                                <button class="btn btn-primary btn-lg sb-cta" on:click=move |_| n2("/history", Default::default())>
                                                    "View Gallery"
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <p class="proc-hint">"Processing your image. Do not close this page."</p>
                                            }.into_any()
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }.into_any()}
            </aside>
        </div>

        <style>
            "
            /* ── Shell ── */
            .editor-shell {
                display: flex;
                height: calc(100vh - 72px);
                overflow: hidden;
                background: transparent;
            }

            /* ── Canvas ── */
            .editor-canvas-area {
                flex: 1;
                min-width: 0;
                position: relative;
                display: flex;
                align-items: center;
                justify-content: center;
                overflow: hidden;
            }

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
                padding: 2px;
                border-radius: calc(var(--radius-lg) + 2px);
                background: linear-gradient(140deg,
                    hsl(var(--accent) / 0.12) 0%,
                    transparent 45%,
                    hsl(var(--accent) / 0.06) 100%
                );
                cursor: pointer;
                transition: background 0.3s;
            }
            .drop-zone-outer:hover {
                background: linear-gradient(140deg,
                    hsl(var(--accent) / 0.26) 0%,
                    hsl(var(--accent) / 0.04) 45%,
                    hsl(var(--accent) / 0.15) 100%
                );
            }
            .drop-zone-inner {
                background: var(--glass);
                backdrop-filter: blur(20px) saturate(140%);
                border-radius: var(--radius-lg);
                border: 1px dashed rgba(255,255,255,0.08);
                padding: 3.5rem 4.5rem;
                display: flex; flex-direction: column;
                align-items: center; gap: var(--s-3);
                text-align: center;
                transition: border-color 0.25s;
                user-select: none;
            }
            .drop-zone-outer:hover .drop-zone-inner { border-color: hsl(var(--accent) / 0.22); }
            .dz-icon-wrap {
                width: 64px; height: 64px;
                background: hsl(var(--accent) / 0.07);
                border: 1px solid hsl(var(--accent) / 0.13);
                border-radius: 18px;
                display: flex; align-items: center; justify-content: center;
                color: hsl(var(--accent) / 0.65);
                margin-bottom: var(--s-2);
                transition: all 0.25s;
            }
            .drop-zone-outer:hover .dz-icon-wrap {
                background: hsl(var(--accent) / 0.12);
                border-color: hsl(var(--accent) / 0.28);
                color: hsl(var(--accent));
            }
            .dz-title {
                font-size: 1.125rem; font-weight: 700;
                color: hsl(var(--text));
                font-family: var(--font-heading);
                letter-spacing: -0.02em;
            }
            .dz-sub { font-size: 0.875rem; color: hsl(var(--text-dim)); font-weight: 400; }
            .dz-formats {
                margin-top: var(--s-3);
                font-size: 0.5625rem; font-weight: 700;
                color: hsl(var(--text-dim) / 0.35);
                letter-spacing: 0.14em; text-transform: uppercase;
                font-family: var(--font-mono);
            }

            /* Drag overlay */
            .drag-overlay {
                position: absolute; inset: 0;
                background: hsl(var(--bg) / 0.75);
                backdrop-filter: blur(16px);
                display: flex; flex-direction: column;
                align-items: center; justify-content: center;
                gap: var(--s-3); color: hsl(var(--accent));
                border: 2px dashed hsl(var(--accent) / 0.5);
                z-index: 50;
                font-size: 0.875rem; font-weight: 700;
            }

            /* ── Sidebar ── */
            .editor-sidebar {
                width: 340px; flex-shrink: 0;
                border-left: 1px solid var(--glass-border);
                background: hsl(var(--surface) / 0.7);
                backdrop-filter: blur(24px) saturate(160%);
                display: flex; flex-direction: column;
                overflow: hidden;
            }

            .sidebar-inner {
                display: flex; flex-direction: column;
                height: 100%; overflow: hidden;
            }

            .sb-scroll {
                flex: 1; overflow-y: auto;
                padding: var(--s-6);
                display: flex; flex-direction: column;
                gap: var(--s-4);
            }
            .sb-scroll::-webkit-scrollbar { width: 3px; }
            .sb-scroll::-webkit-scrollbar-thumb {
                background: hsl(var(--border) / 0.35); border-radius: 3px;
            }

            /* Error */
            .sb-error {
                display: flex; align-items: center; gap: var(--s-3);
                padding: var(--s-3) var(--s-4);
                background: hsl(var(--error) / 0.08);
                border: 1px solid hsl(var(--error) / 0.2);
                border-radius: var(--radius-md);
                color: hsl(var(--error));
                font-size: 0.8125rem; font-weight: 600;
            }

            /* Cards — use global .card + editor-card-body for internal padding */
            .editor-card { padding: 0 !important; }
            .editor-card-body { padding: var(--s-6); }

            /* Resolution grid */
            .res-grid {
                display: grid; grid-template-columns: 1fr 1fr;
                gap: var(--s-3);
            }
            /* Big numeral inside pack-item for resolution */
            .res-big-num {
                font-family: var(--font-heading);
                font-size: 1.5rem; font-weight: 800;
                color: hsl(var(--text));
                line-height: 1; letter-spacing: -0.04em;
            }

            /* Segmented control */
            .seg-control {
                display: flex;
                background: hsl(var(--surface-raised));
                border: 1px solid hsl(var(--border));
                border-radius: var(--radius-md);
                padding: 3px; gap: 3px;
            }
            .seg-control button {
                flex: 1; border: none;
                background: transparent;
                color: hsl(var(--text-dim));
                padding: 8px 0;
                font-size: 0.8125rem; font-weight: 600;
                border-radius: 8px; cursor: pointer;
                transition: all 0.18s;
                font-family: var(--font-body);
            }
            .seg-control button.active {
                background: hsl(var(--surface-bright));
                color: hsl(var(--text));
                box-shadow: var(--shadow-sm);
            }

            /* Field helpers */
            .sb-field { display: flex; flex-direction: column; gap: var(--s-2); }
            .sb-label {
                font-size: 0.6875rem; font-weight: 700;
                color: hsl(var(--text-dim));
            }
            .sb-label-row {
                display: flex; justify-content: space-between; align-items: center;
            }
            .sb-val-badge {
                font-family: var(--font-mono);
                font-size: 0.6875rem; font-weight: 700;
                color: hsl(var(--accent));
                background: hsl(var(--accent) / 0.08);
                padding: 2px 8px; border-radius: 4px;
                border: 1px solid hsl(var(--accent) / 0.15);
            }
            .sb-val-badge.mono { font-family: var(--font-mono); }

            /* Slider */
            .slider-wrap { display: flex; flex-direction: column; gap: 6px; }
            .studio-slider {
                width: 100%; height: 3px;
                background: hsl(var(--border) / 0.5);
                border-radius: 3px; appearance: none;
                outline: none; cursor: pointer; border: none; padding: 0;
            }
            .studio-slider::-webkit-slider-thumb {
                appearance: none;
                width: 16px; height: 16px;
                background: hsl(var(--text));
                border-radius: 50%; cursor: pointer;
                border: 2px solid hsl(var(--bg));
                box-shadow: 0 0 0 2px hsl(var(--accent) / 0.2), 0 2px 6px rgba(0,0,0,0.4);
                transition: box-shadow 0.15s;
            }
            .studio-slider::-webkit-slider-thumb:hover {
                box-shadow: 0 0 0 4px hsl(var(--accent) / 0.2);
            }
            .slider-ends {
                display: flex; justify-content: space-between;
                font-size: 0.625rem; font-weight: 600;
                color: hsl(var(--text-dim) / 0.35);
            }

            /* Input */
            .sb-input {
                flex: 1;
                background: hsl(var(--surface-raised));
                border: 1px solid hsl(var(--border));
                border-radius: var(--radius-md);
                padding: 9px 12px;
                color: hsl(var(--text));
                font-size: 0.8125rem;
                font-family: var(--font-body);
                transition: all 0.18s;
                width: 100%;
            }
            .sb-input::placeholder { color: hsl(var(--text-dim) / 0.3); }
            .sb-input:focus {
                border-color: hsl(var(--accent) / 0.5);
                background: hsl(var(--surface-bright));
                outline: none;
                box-shadow: 0 0 0 3px hsl(var(--accent) / 0.08);
            }

            /* Seed row */
            .seed-row { display: flex; gap: var(--s-2); }
            .seed-rng-btn {
                width: 40px; flex-shrink: 0;
                background: hsl(var(--surface-raised));
                border: 1px solid hsl(var(--border));
                border-radius: var(--radius-md);
                color: hsl(var(--text-dim));
                cursor: pointer;
                display: flex; align-items: center; justify-content: center;
                transition: all 0.18s;
            }
            .seed-rng-btn:hover {
                background: hsl(var(--surface-bright));
                border-color: hsl(var(--accent) / 0.4);
                color: hsl(var(--text));
            }

            /* DOF toggle — reuses pack-item, just needs the toggle switch */
            .dof-row { align-items: center; }
            .toggle-track {
                width: 34px; height: 18px; flex-shrink: 0;
                background: hsl(var(--surface-raised));
                border: 1px solid hsl(var(--border));
                border-radius: 100px; padding: 3px;
                transition: all 0.25s;
            }
            .pack-item.active .toggle-track {
                background: hsl(var(--accent) / 0.15);
                border-color: hsl(var(--accent) / 0.5);
            }
            .toggle-thumb {
                width: 12px; height: 12px;
                background: hsl(var(--text-dim) / 0.4);
                border-radius: 50%;
                transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .pack-item.active .toggle-thumb {
                transform: translateX(16px);
                background: hsl(var(--accent));
                box-shadow: 0 0 6px hsl(var(--accent) / 0.5);
            }

            /* Select */
            .select-wrap { position: relative; }
            .sb-select {
                width: 100%;
                background: hsl(var(--surface-raised));
                border: 1px solid hsl(var(--border));
                border-radius: var(--radius-md);
                padding: 10px 36px 10px 12px;
                color: hsl(var(--text));
                font-size: 0.8125rem;
                font-family: var(--font-body); font-weight: 500;
                appearance: none; cursor: pointer; outline: none;
                transition: all 0.18s;
            }
            .sb-select:focus {
                border-color: hsl(var(--accent) / 0.4);
                box-shadow: 0 0 0 3px hsl(var(--accent) / 0.07);
            }

            /* Footer CTA */
            .sb-footer {
                padding: var(--s-4) var(--s-6) var(--s-5);
                border-top: 1px solid hsl(var(--border-muted));
                background: hsl(var(--surface) / 0.5);
                flex-shrink: 0;
            }
            .sb-cta {
                width: 100% !important;
                position: relative;
                gap: 10px;
                justify-content: center;
            }
            .sb-cta:disabled { opacity: 0.35; cursor: not-allowed; transform: none !important; box-shadow: none !important; }
            .sb-cta-credit {
                font-size: 0.6875rem;
                font-weight: 600;
                opacity: 0.65;
                padding: 2px 8px;
                border-radius: 4px;
                background: rgba(0,0,0,0.15);
                margin-left: auto;
            }

            /* ── Processing panel ── */
            .processing-panel { justify-content: space-between; }
            .proc-body {
                flex: 1; display: flex; flex-direction: column;
                align-items: center; justify-content: center;
                padding: var(--s-10) var(--s-8);
                text-align: center; gap: var(--s-4);
            }
            .proc-icon {
                width: 72px; height: 72px;
                background: hsl(var(--accent) / 0.07);
                border: 1px solid hsl(var(--accent) / 0.14);
                border-radius: 50%;
                display: flex; align-items: center; justify-content: center;
                color: hsl(var(--accent));
                margin-bottom: var(--s-2);
            }
            .proc-stage {
                font-size: 0.5625rem; font-weight: 900;
                letter-spacing: 0.2em; text-transform: uppercase;
                color: hsl(var(--accent));
                background: hsl(var(--accent) / 0.08);
                border: 1px solid hsl(var(--accent) / 0.15);
                padding: 3px 12px; border-radius: 100px;
            }
            .proc-title {
                font-size: 1.375rem; font-weight: 800;
                letter-spacing: -0.03em;
                font-family: var(--font-heading);
            }
            .proc-desc {
                font-size: 0.875rem; color: hsl(var(--text-dim));
                max-width: 220px; line-height: 1.5;
            }
            .proc-bar-wrap { width: 100%; max-width: 220px; }
            .proc-bar-track {
                height: 2px; background: hsl(var(--border) / 0.3);
                border-radius: 2px; overflow: hidden; margin-bottom: 6px;
            }
            .proc-bar-fill {
                height: 100%; background: hsl(var(--accent));
                width: 100%; transform: translateX(-100%);
                animation: proc-slide 2s infinite ease-in-out;
            }
            @keyframes proc-slide {
                0%   { transform: translateX(-100%); }
                100% { transform: translateX(100%); }
            }
            .proc-bar-labels {
                display: flex; justify-content: space-between;
                font-size: 0.5625rem; font-weight: 700;
                color: hsl(var(--text-dim) / 0.25);
                letter-spacing: 0.1em; text-transform: uppercase;
            }
            .proc-latency {
                display: flex; align-items: center; gap: 8px;
                font-size: 0.75rem; color: hsl(var(--text-dim));
                padding: 8px 16px;
                background: hsl(var(--surface-raised));
                border-radius: var(--radius-md);
                border: 1px solid hsl(var(--border));
            }
            .proc-latency-val {
                font-family: var(--font-mono); font-weight: 800;
                color: hsl(var(--accent));
            }
            .proc-footer {
                padding: var(--s-6);
                border-top: 1px solid hsl(var(--border-muted));
            }
            .proc-hint {
                font-size: 0.75rem; color: hsl(var(--text-dim));
                text-align: center; line-height: 1.5;
            }
            "
        </style>
    }
}

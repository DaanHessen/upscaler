use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle, LogOut, Download, Info};
use crate::text::TXT;
use wasm_bindgen::JsCast;

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();
    
    // Internal state for the upscale engine
    let (processing_job, set_processing_job) = signal(Option::<uuid::Uuid>::None);
    let (engine_status, set_engine_status) = signal(Option::<PollResponse>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_dragging, set_is_dragging) = signal(false);

    // Watch for job creation and start polling
    Effect::new(move |_| {
        if let Some(job_id) = processing_job.get() {
            let token = auth.session.get().map(|s| s.access_token);
            let state = global_state;
            
            // Polling task
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
                            if r.status == "FAILED" {
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
                    set_error_msg.set(Some("Insufficient credits available.".to_string()));
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
                    Err(e) => {
                        set_error_msg.set(Some(format!("Upload failed: {}", e)));
                    }
                }
            });
        }
    };

    let stage_info = move || {
        match engine_status.get().map(|s| s.status) {
            Some(s) if s == "PENDING" => ("QUEUE", "System Ready", "Analyzing infrastructure availability..."),
            Some(s) if s == "PROCESSING" => ("ACTIVE", "Reconstructing", "Gemini Vision is synthesizing high-freq details."),
            Some(s) if s == "COMPLETED" => ("FINISH", "Studio Export", "Enhancement complete. Ready for download."),
            _ => ("IDLE", "Standby", "Awaiting engine handshake.")
        }
    };

    let nav_home = navigate.clone();
    let nav_history = navigate.clone();

    view! {
        <div class="editor-shell fade-in">
            // --- Primary Canvas ---
            <div class="editor-main"
                 on:dragover=move |ev| { ev.prevent_default(); set_is_dragging.set(true); }
                 on:dragleave=move |_| set_is_dragging.set(false)
                 on:drop=on_drop
            >
                // Page Title Header (Matches History/Billing)
                <div class="page-header" style="position: absolute; top: var(--s-8); left: var(--s-12); z-index: 50; width: auto; background: transparent; padding: 0;">
                    <div class="header-main">
                        <h1 class="stagger-1 text-gradient" style="font-size: 2.25rem;">{TXT.editor_page_title}</h1>
                        <p class="muted stagger-2">{TXT.editor_page_subtitle}</p>
                    </div>
                </div>

                <div class="editor-canvas">
                    <div class="canvas-grid"></div>
                    
                    <div class="asset-frame">
                        {move || {
                            let nav = nav_home.clone();
                            let img_url = global_state.preview_base64.get();
                            match img_url {
                                Some(url) => view! {
                                    <div class="asset-wrapper fade-in">
                                        <img src=url class="studio-asset" alt="Upscale Result" />
                                        <div class="laser-scanner"></div>
                                        <div class="corner-accents">
                                            <div class="corner tl"></div>
                                            <div class="corner tr"></div>
                                            <div class="corner bl"></div>
                                            <div class="corner br"></div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="empty-canvas stagger-1">
                                        <div class="drop-zone-trigger" on:click=move |_| {
                                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                                if let Some(el) = doc.get_element_by_id("hidden_file_input") {
                                                    let html_el: web_sys::HtmlElement = el.unchecked_into();
                                                    html_el.click();
                                                }
                                            }
                                        }>
                                            <ImageIcon size={64} />
                                            <h3>{TXT.editor_empty_title}</h3>
                                            <p>{TXT.editor_empty_desc}</p>
                                        </div>
                                        <input type="file" id="hidden_file_input" style="display: none;" on:change=on_file_input />
                                    </div>
                                }.into_any()
                            }
                        }}

                        // Overlay for dragging
                        {move || is_dragging.get().then(|| view! {
                            <div class="drag-overlay fade-in">
                                <Download size={48} />
                                <h2>"DROP IMAGE TO BEGIN"</h2>
                            </div>
                        })}
                    </div>

                    // --- Canvas Telemetry ---
                    <div class="canvas-telemetry">
                        <div class="telemetry-pill">
                            <span class="label">"DETECTED:"</span>
                            <span class="value accent">{move || global_state.temp_classification.get().unwrap_or_else(|| "NONE".to_string())}</span>
                        </div>
                        {move || processing_job.get().map(|id| view! {
                            <div class="telemetry-pill job-pill">
                                <span class="label">"JOB_ID:"</span>
                                <span class="value font-mono">{id.to_string().chars().take(8).collect::<String>()}</span>
                            </div>
                        })}
                    </div>
                </div>
            </div>

            // --- Right Sidebar: Editor Controls ---
            <aside class="editor-sidebar-wrapper">
                <div class="sidebar-backdrop"></div>
                <div class="editor-sidebar">
                    
                    {move || {
                        let job = processing_job.get();
                        let nav = nav_history.clone();
                        match job {
                            None => view! {
                                // --- SETTINGS MODE ---
                                <div class="sidebar-content fade-in">
                                    <div class="sidebar-header">
                                        <div class="card-tag">
                                            <Settings size={12} />
                                            <span>"UPSCALE PARAMETERS"</span>
                                        </div>
                                        <h2 class="sidebar-title">{TXT.editor_sidebar_title}</h2>
                                    </div>

                                    <div class="sidebar-scrollable">
                                        // Error message
                                        {move || error_msg.get().map(|msg| view! {
                                            <div class="sidebar-error animate-slide-up">
                                                <AlertCircle size={14} />
                                                <span>{msg}</span>
                                            </div>
                                        })}

                                        // Resolution
                                        <div class="input-group">
                                            <div class="label-row">
                                                <label class="group-label">{TXT.label_resolution}</label>
                                                <div class="tooltip-wrapper">
                                                    <Info size={12} />
                                                    <div class="tooltip">{TXT.desc_resolution}</div>
                                                </div>
                                            </div>
                                            <div class="resolution-grid">
                                                <div 
                                                    class=move || if global_state.quality.get() == "2K" { "res-tile active" } else { "res-tile" }
                                                    on:click=move |_| global_state.set_quality.set("2K".to_string())
                                                >
                                                    <span class="res-num">"2K"</span>
                                                    <span class="res-desc">"HD RESTORE"</span>
                                                    <div class="res-tag">"2 CREDITS"</div>
                                                </div>
                                                <div 
                                                    class=move || if global_state.quality.get() == "4K" { "res-tile active" } else { "res-tile" }
                                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                                >
                                                    <span class="res-num">"4K"</span>
                                                    <span class="res-desc">"ULTRA HD"</span>
                                                    <div class="res-tag">"4 CREDITS"</div>
                                                </div>
                                            </div>
                                        </div>

                                        // Style
                                        <div class="input-group">
                                            <div class="label-row">
                                                <label class="group-label">{TXT.label_style}</label>
                                                <div class="tooltip-wrapper">
                                                    <Info size={12} />
                                                    <div class="tooltip">{TXT.desc_style}</div>
                                                </div>
                                            </div>
                                            <div class="style-switcher">
                                                <button 
                                                    class:active=move || global_state.style.get() == "PHOTOGRAPHY"
                                                    on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                                >
                                                    "PHOTOGRAPHY"
                                                </button>
                                                <button 
                                                    class:active=move || global_state.style.get() == "ILLUSTRATION"
                                                    on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                                >
                                                    "ILLUSTRATION"
                                                </button>
                                            </div>
                                        </div>

                                        // Creativity (was Creative Drift)
                                        <div class="input-group">
                                            <div class="label-row">
                                                <div style="display: flex; gap: var(--s-2); align-items: center;">
                                                    <label class="group-label">{TXT.label_creativity}</label>
                                                    <div class="tooltip-wrapper">
                                                        <Info size={12} />
                                                        <div class="tooltip">{TXT.desc_creativity}</div>
                                                    </div>
                                                </div>
                                                <span class="drift-val">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                            </div>
                                            <div class="slider-wrapper">
                                                <input 
                                                    type="range" min="0.0" max="2.0" step="0.1"
                                                    prop:value=move || global_state.temperature.get().to_string()
                                                    on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                                />
                                                <div class="slider-labels">
                                                    <span>"STRICT"</span>
                                                    <span>"CREATIVE"</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Seed
                                        <div class="input-group">
                                            <div class="label-row">
                                                <div style="display: flex; gap: var(--s-2); align-items: center;">
                                                    <label class="group-label">{TXT.label_seed}</label>
                                                    <div class="tooltip-wrapper">
                                                        <Info size={12} />
                                                        <div class="tooltip">{TXT.desc_seed}</div>
                                                    </div>
                                                </div>
                                                <span class="drift-val-pill">
                                                    {move || global_state.seed.get().map(|s| s.to_string()).unwrap_or_else(|| "AUTO".to_string())}
                                                </span>
                                            </div>
                                            <div class="seed-control">
                                                <input 
                                                    type="number" 
                                                    class="studio-input seed-input"
                                                    placeholder="Automatic"
                                                    prop:value=move || global_state.seed.get().map(|s| s.to_string()).unwrap_or_default()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        if val.is_empty() {
                                                            global_state.set_seed.set(None);
                                                        } else if let Ok(s) = val.parse::<u32>() {
                                                            global_state.set_seed.set(Some(s));
                                                        }
                                                    }
                                                />
                                                <div class="seed-actions">
                                                    <button 
                                                        class="seed-btn" 
                                                        on:click=move |_| {
                                                            let val = (js_sys::Math::random() * (u32::MAX as f64)) as u32;
                                                            global_state.set_seed.set(Some(val));
                                                        }
                                                    >
                                                        <RefreshCw size={14} />
                                                    </button>
                                                    <button 
                                                        class="seed-btn"
                                                        on:click=move |_| global_state.set_seed.set(None)
                                                    >
                                                        <LogOut size={14} custom_style="transform: rotate(90deg)".to_string() />
                                                    </button>
                                                </div>
                                            </div>
                                        </div>

                                        // Advanced
                                        <div class="input-group">
                                            <div class="label-row">
                                                <label class="group-label">{TXT.label_locks}</label>
                                                <div class="tooltip-wrapper">
                                                    <Info size={12} />
                                                    <div class="tooltip">{TXT.desc_locks}</div>
                                                </div>
                                            </div>
                                            <div 
                                                class="advanced-toggle" 
                                                class:active=move || global_state.keep_depth_of_field.get()
                                                on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                            >
                                                <div class="toggle-icon"><Target size={14} /></div>
                                                <div class="toggle-meta">
                                                    <span class="toggle-title">"DEPTH OF FIELD LOCK"</span>
                                                    <span class="toggle-sub">"Preserves original focal planes"</span>
                                                </div>
                                                <div class="toggle-check">
                                                    <div class="check-dot"></div>
                                                </div>
                                            </div>
                                        </div>

                                        // Lighting
                                        <div class="input-group">
                                            <div class="label-row">
                                                <label class="group-label">{TXT.label_lighting}</label>
                                                <div class="tooltip-wrapper">
                                                    <Info size={12} />
                                                    <div class="tooltip">{TXT.desc_lighting}</div>
                                                </div>
                                            </div>
                                            <div class="studio-select-wrapper">
                                                <select 
                                                    class="studio-select"
                                                    on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                                    prop:value=move || global_state.lighting.get()
                                                >
                                                    <option value="Original">"ORIGINAL LIGHTING"</option>
                                                    <option value="Studio">"STUDIO LIGHTING"</option>
                                                    <option value="Cinematic">"CINEMATIC SHADOWS"</option>
                                                    <option value="Vivid">"VIVID DYNAMICS"</option>
                                                    <option value="Natural">"NATURAL OVERCAST"</option>
                                                </select>
                                                <div class="select-arrow"></div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="sidebar-footer">
                                        <button 
                                            class="editor-submit-btn"
                                            on:click=handle_upscale
                                            disabled=move || global_state.temp_file.get().is_none()
                                        >
                                            <div class="btn-inner">
                                                <Zap size={18} />
                                                <span>"INITIATE UPSCALE"</span>
                                                <div class="btn-cost">
                                                    {move || if global_state.quality.get() == "4K" { "4 CREDITS" } else { "2 CREDITS" }}
                                                </div>
                                            </div>
                                            <div class="btn-glow"></div>
                                        </button>
                                    </div>
                                </div>
                            }.into_any(),

                            Some(_) => {
                                let n = nav.clone();
                                view! {
                                    // --- PROCESSING MODE ---
                                    <div class="sidebar-content processing-state fade-in">
                                        <div class="sidebar-header">
                                            <div class="card-tag">
                                                <RefreshCw size={12} custom_style="animation: spin 2s linear infinite".to_string() />
                                                <span>"ENGINE ACTIVE"</span>
                                            </div>
                                            <h2 class="sidebar-title">"Upscaling..."</h2>
                                        </div>

                                        <div class="processing-vitals">
                                            <div class="pulse-ring">
                                                <div class="ring r1"></div>
                                                <div class="ring r2"></div>
                                                <Zap size={32} />
                                            </div>
                                            
                                            <div class="status-box">
                                                <span class="status-tag">{move || stage_info().0}</span>
                                                <h3 class="status-title">{move || stage_info().1}</h3>
                                                <p class="status-desc">{move || stage_info().2}</p>
                                            </div>

                                            <div class="progress-container">
                                                <div class="progress-bar-rail">
                                                    <div class="progress-bar-fill active"></div>
                                                </div>
                                                <div class="progress-labels">
                                                    <span>"RECONSTRUCTION"</span>
                                                    <span class="animate-pulse">"RUNNING"</span>
                                                </div>
                                            </div>

                                            {move || engine_status.get().and_then(|s| s.latency_ms).map(|ms| view! {
                                                <div class="latency-telemetry fade-in">
                                                    <span class="latency-label">"PROCESSING DURATION:"</span>
                                                    <span class="latency-value">{format!("{:.1}s", ms as f32 / 1000.0)}</span>
                                                </div>
                                            })}
                                        </div>

                                        <div class="sidebar-note">
                                            <p>"Your image is being processed by the Gemini Vision infrastructure."</p>
                                            {move || if engine_status.get().map(|s| s.status == "COMPLETED").unwrap_or(false) {
                                                let n2 = n.clone();
                                                view! {
                                                    <button class="btn btn-primary" style="margin-top: var(--s-4); width: 100%;" on:click=move |_| n2("/history", Default::default())>"VIEW IN GALLERY"</button>
                                                }.into_any()
                                            } else {
                                                view! { <p style="font-size: 0.75rem; opacity: 0.5;">"Do not close this tab until completion."</p> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }.into_any()}
                </div>
            </aside>
        </div>

        <style>
            "/* ─── EDITOR SHELL ─── */
            .editor-shell {
                display: flex;
                height: calc(100vh - 72px);
                background: #050505;
                overflow: hidden;
            }

            .editor-main {
                flex: 1;
                position: relative;
                overflow: hidden;
            }

            .editor-canvas {
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                position: relative;
                padding: var(--s-20) var(--s-12) var(--s-12) var(--s-12);
            }

            .canvas-grid {
                position: absolute;
                inset: 0;
                background-image: 
                    linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px),
                    linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px);
                background-size: 40px 40px;
                mask-image: radial-gradient(circle at center, black, transparent 80%);
            }

            .asset-frame {
                position: relative;
                max-width: 85%;
                max-height: 85%;
                z-index: 10;
                transition: transform 0.3s;
            }

            .studio-asset {
                display: block;
                max-width: 100%;
                max-height: 65vh;
                border-radius: 4px;
                box-shadow: 0 30px 60px rgba(0,0,0,0.8), 0 0 0 1px rgba(255,255,255,0.05);
            }

            .asset-wrapper { position: relative; }

            .laser-scanner {
                position: absolute;
                top: 0; left: 0; right: 0; height: 2px;
                background: linear-gradient(90deg, transparent, hsl(var(--accent)), transparent);
                box-shadow: 0 0 15px hsl(var(--accent));
                animation: scan 4s ease-in-out infinite;
                z-index: 20;
            }

            @keyframes scan {
                0%, 100% { top: 0%; opacity: 0; }
                10%, 90% { opacity: 1; }
                50% { top: 100%; }
            }

            .corner-accents .corner {
                position: absolute; width: 20px; height: 20px;
                border: 2px solid hsl(var(--accent) / 0.3);
                z-index: 25;
            }
            .corner.tl { top: -10px; left: -10px; border-right: 0; border-bottom: 0; }
            .corner.tr { top: -10px; right: -10px; border-left: 0; border-bottom: 0; }
            .corner.bl { bottom: -10px; left: -10px; border-right: 0; border-top: 0; }
            .corner.br { bottom: -10px; right: -10px; border-left: 0; border-top: 0; }

            .empty-canvas {
                text-align: center; color: hsl(var(--text-dim));
            }
            .drop-zone-trigger {
                cursor: pointer; padding: var(--s-12); border: 2px dashed rgba(255,255,255,0.05);
                border-radius: var(--radius-lg); transition: all 0.3s;
            }
            .drop-zone-trigger:hover { background: rgba(255,255,255,0.02); border-color: hsl(var(--accent) / 0.3); }
            .empty-canvas h3 { font-size: 1.5rem; margin: var(--s-4) 0 var(--s-2); color: white; }
            .empty-canvas p { font-size: 0.875rem; margin-bottom: var(--s-6); }

            .drag-overlay {
                position: absolute; inset: -20px; background: hsla(var(--accent-h), var(--accent-s), 5%, 0.9);
                backdrop-filter: blur(10px); z-index: 100; border: 2px solid hsl(var(--accent));
                border-radius: var(--radius-lg); display: flex; flex-direction: column; align-items: center; justify-content: center;
                gap: var(--s-4); color: hsl(var(--accent));
            }

            .canvas-telemetry {
                position: absolute; bottom: var(--s-8); left: 50%; transform: translateX(-50%);
                display: flex; gap: var(--s-4); z-index: 30;
            }
            .telemetry-pill {
                background: rgba(0,0,0,0.6); backdrop-filter: blur(10px);
                border: 1px solid rgba(255,255,255,0.05);
                padding: 6px 12px; border-radius: 100px;
                display: flex; gap: 8px; font-size: 0.625rem; font-weight: 800; letter-spacing: 0.05em;
            }
            .telemetry-pill .label { color: hsl(var(--text-dim) / 0.6); }
            .telemetry-pill .value { color: white; }
            .telemetry-pill .value.accent { color: hsl(var(--accent)); }

            /* ─── SIDEBAR ─── */
            .editor-sidebar-wrapper {
                width: 400px; position: relative; border-left: 1px solid rgba(255,255,255,0.03);
            }
            .sidebar-backdrop {
                position: absolute; inset: 0; background: #080808; opacity: 0.8;
            }
            .editor-sidebar {
                position: relative; height: 100%; display: flex; flex-direction: column; z-index: 10;
            }
            .sidebar-content {
                display: flex; flex-direction: column; height: 100%;
            }
            .sidebar-header { padding: var(--s-8) var(--s-8) var(--s-6); }
            .sidebar-title { font-size: 1.75rem; font-weight: 800; margin-top: var(--s-2); letter-spacing: -0.03em; }

            .sidebar-scrollable {
                flex: 1; overflow-y: auto; padding: 0 var(--s-8) var(--s-8);
            }
            
            .sidebar-scrollable::-webkit-scrollbar { width: 3px; }
            .sidebar-scrollable::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 10px; }

            .input-group { margin-bottom: var(--s-10); }
            .group-label { 
                font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text-dim)); 
                letter-spacing: 0.12em; margin-bottom: 0px; display: flex; align-items: center; gap: var(--s-3); 
            }

            .tooltip-wrapper { position: relative; display: flex; align-items: center; color: hsl(var(--text-dim) / 0.4); cursor: help; }
            .tooltip { 
                position: absolute; bottom: 100%; left: 50%; transform: translateX(-50%) translateY(-10px);
                background: #1a1a1a; color: white; padding: 10px; border-radius: 8px; width: 200px;
                font-size: 0.6875rem; font-weight: 600; line-height: 1.4; opacity: 0; pointer-events: none;
                transition: all 0.2s; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 10px 30px rgba(0,0,0,0.5);
                z-index: 1000; text-align: center;
            }
            .tooltip-wrapper:hover .tooltip { opacity: 1; transform: translateX(-50%) translateY(-5px); }

            .resolution-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); }
            .res-tile {
                background: #0d0d0d; border: 1px solid rgba(255,255,255,0.04);
                border-radius: var(--radius-lg); padding: var(--s-6);
                cursor: pointer; transition: all 0.3s; position: relative; overflow: hidden;
            }
            .res-tile:hover { background: #111; border-color: rgba(255,255,255,0.1); }
            .res-tile.active { background: #151515; border-color: hsl(var(--accent)); box-shadow: 0 0 20px hsl(var(--accent) / 0.15); }
            .res-num { display: block; font-size: 1.25rem; font-weight: 900; color: white; }
            .res-desc { display: block; font-size: 0.625rem; font-weight: 700; color: hsl(var(--text-dim)); margin-top: 2px; }
            .res-tag { 
                position: absolute; top: 0; right: 0; background: rgba(255,255,255,0.03); 
                padding: 4px 8px; font-size: 0.45rem; font-weight: 900; border-bottom-left-radius: 8px; color: hsl(var(--accent) / 0.6);
            }

            .style-switcher { 
                display: flex; background: #0d0d0d; padding: 4px; border-radius: var(--radius-lg); 
                border: 1px solid rgba(255,255,255,0.04);
            }
            .style-switcher button {
                flex: 1; border: none; background: transparent; color: hsl(var(--text-dim) / 0.6);
                padding: var(--s-3) 0; font-size: 0.625rem; font-weight: 900; border-radius: calc(var(--radius-lg) - 4px);
                cursor: pointer; transition: all 0.3s; letter-spacing: 0.05em;
            }
            .style-switcher button.active { background: #1a1a1a; color: hsl(var(--text)); box-shadow: 0 4px 12px rgba(0,0,0,0.3); }

            .label-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--s-4); min-height: 20px; }
            .drift-val { background: rgba(255,255,255,0.05); color: white; font-size: 0.75rem; font-weight: 800; font-family: var(--font-mono); padding: 2px 8px; border-radius: 4px; }
            .drift-val-pill { background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); font-size: 0.75rem; font-weight: 900; font-family: var(--font-mono); padding: 2px 10px; border-radius: 6px; border: 1px solid hsl(var(--accent) / 0.1); }

            .slider-wrapper { padding: var(--s-2) 0; }
            .slider-wrapper input { width: 100%; height: 6px; background: #111; border-radius: 10px; appearance: none; outline: none; }
            .slider-wrapper input::-webkit-slider-thumb { 
                appearance: none; width: 18px; height: 18px; background: white; border-radius: 50%; cursor: pointer;
                box-shadow: 0 0 10px rgba(255,255,255,0.3);
            }
            .slider-labels { display: flex; justify-content: space-between; font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim) / 0.3); letter-spacing: 0.1em; }

            .seed-control { display: flex; gap: var(--s-2); margin-top: var(--s-2); }
            .seed-input { 
                flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 0.75rem; text-align: center; 
                background: #0d0d0d; border: 1px solid rgba(255,255,255,0.04); border-radius: var(--radius-md);
                color: white; padding: var(--s-2); transition: all 0.2s;
            }
            .seed-input:focus { border-color: hsl(var(--accent) / 0.4); outline: none; background: #111; }
            .seed-actions { display: flex; gap: 4px; }
            .seed-btn { 
                background: #0d0d0d; border: 1px solid rgba(255,255,255,0.04); 
                width: 38px; border-radius: var(--radius-md); color: hsl(var(--text-dim) / 0.5); 
                cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center;
            }
            .seed-btn:hover { background: #111; color: white; border-color: rgba(255,255,255,0.1); }
            .input-hint { font-size: 0.625rem; color: hsl(var(--text-dim) / 0.4); margin-top: var(--s-3); line-height: 1.4; font-weight: 600; }

            .advanced-toggle {
                background: #0d0d0d; border: 1px solid rgba(255,255,255,0.04);
                padding: var(--s-4); border-radius: var(--radius-lg);
                display: flex; align-items: center; gap: var(--s-4); cursor: pointer; transition: all 0.3s;
            }
            .advanced-toggle.active { border-color: hsl(var(--accent) / 0.5); background: hsl(var(--accent) / 0.02); }
            .toggle-icon { width: 32px; height: 32px; background: #151515; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: hsl(var(--text-dim)); transition: all 0.3s; }
            .advanced-toggle.active .toggle-icon { background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); }
            .toggle-meta { flex: 1; }
            .toggle-title { display: block; font-size: 0.75rem; font-weight: 800; color: white; }
            .toggle-sub { font-size: 0.625rem; color: hsl(var(--text-dim)); }
            .toggle-check { width: 40px; height: 20px; background: #151515; border-radius: 100px; position: relative; padding: 3px; }
            .check-dot { width: 14px; height: 14px; background: #333; border-radius: 50%; transition: all 0.3s; }
            .advanced-toggle.active .check-dot { transform: translateX(20px); background: hsl(var(--accent)); box-shadow: 0 0 8px hsl(var(--accent)); }

            .studio-select-wrapper { position: relative; }
            .studio-select {
                width: 100%; background: #0d0d0d; border: 1px solid rgba(255,255,255,0.04);
                padding: var(--s-3) var(--s-4); border-radius: var(--radius-md); color: white; font-size: 0.75rem;
                font-weight: 700; outline: none; appearance: none; cursor: pointer;
            }
            .select-arrow { 
                position: absolute; right: 12px; top: 50%; transform: translateY(-50%);
                width: 8px; height: 8px; border-right: 2px solid rgba(255,255,255,0.2); 
                border-bottom: 2px solid rgba(255,255,255,0.2); transform: translateY(-70%) rotate(45deg);
                pointer-events: none;
            }

            .sidebar-footer { padding: var(--s-8); border-top: 1px solid rgba(255,255,255,0.03); }
            .editor-submit-btn {
                width: 100%; height: 56px; background: hsl(var(--accent)); border: none; border-radius: var(--radius-lg);
                cursor: pointer; position: relative; overflow: hidden; transition: all 0.3s;
            }
            .editor-submit-btn:disabled { opacity: 0.3; cursor: not_allowed; filter: grayscale(1); }
            .btn-inner { position: relative; z-index: 2; display: flex; align-items: center; justify-content: center; gap: var(--s-3); color: white; font-weight: 900; letter-spacing: 0.05em; font-size: 0.8125rem; }
            .btn-cost { background: rgba(0,0,0,0.2); padding: 4px 8px; border-radius: 4px; font-size: 0.55rem; font-weight: 900; }
            .btn-glow { position: absolute; inset: 0; background: radial-gradient(circle at center, rgba(255,255,255,0.2), transparent 70%); opacity: 0; transition: opacity 0.3s; }
            .editor-submit-btn:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 10px 20px hsl(var(--accent) / 0.3); }
            .editor-submit-btn:hover:not(:disabled) .btn-glow { opacity: 1; }

            /* ─── PROCESSING STATE ─── */
            .processing-state { background: linear-gradient(180deg, #0a0a0a, #050505); }
            .processing-vitals { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: var(--s-10); }
            
            .pulse-ring { position: relative; width: 120px; height: 120px; display: flex; align-items: center; justify-content: center; color: hsl(var(--accent)); margin-bottom: var(--s-10); }
            .ring { position: absolute; border: 2px solid hsl(var(--accent) / 0.2); border-radius: 50%; opacity: 0; }
            .ring.r1 { inset: 0; animation: pulse-ring 2s infinite; }
            .ring.r2 { inset: 10px; animation: pulse-ring 2s infinite 0.5s; }
            @keyframes pulse-ring { 
                0% { transform: scale(0.8); opacity: 0.5; }
                100% { transform: scale(1.3); opacity: 0; }
            }

            .status-box { text-align: center; margin-bottom: var(--s-10); }
            .status-tag { display: inline-block; background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); font-size: 0.625rem; font-weight: 900; padding: 4px 12px; border-radius: 100px; margin-bottom: var(--s-2); }
            .status-title { font-size: 1.25rem; font-weight: 800; margin-bottom: var(--s-2); }
            .status-desc { font-size: 0.75rem; color: hsl(var(--text-dim) / 0.6); max-width: 240px; }

            .progress-container { width: 100%; max-width: 280px; margin-bottom: var(--s-8); }
            .progress-bar-rail { height: 4px; background: rgba(255,255,255,0.05); border-radius: 10px; overflow: hidden; margin-bottom: 8px; }
            .progress-bar-fill { height: 100%; background: hsl(var(--accent)); width: 30%; }
            .progress-bar-fill.active { width: 100%; transform: translateX(-100%); animation: progress-slide 2s infinite ease-in-out; }
            @keyframes progress-slide { 
                0% { transform: translateX(-100%); }
                100% { transform: translateX(100%); }
            }
            .progress-labels { display: flex; justify-content: space-between; font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim) / 0.4); letter-spacing: 0.1em; }

            .latency-telemetry { display: flex; gap: 8px; align-items: center; padding-top: var(--s-4); }
            .latency-label { font-size: 0.625rem; color: hsl(var(--text-dim) / 0.4); font-weight: 800; }
            .latency-value { font-size: 0.625rem; color: hsl(var(--accent)); font-family: var(--font-mono); font-weight: 900; }

            .sidebar-note { padding: var(--s-8); text-align: center; font-size: 0.75rem; color: hsl(var(--text-dim) / 0.5); }
            "
        </style>
    }
}

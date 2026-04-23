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
            Some(s) if s == "PENDING" => ("QUEUE", "System Ready", "Analyzing asset for reconstruction..."),
            Some(s) if s == "PROCESSING" => ("ACTIVE", "Reconstructing", "Gemini Vision is synthesizing high-freq details."),
            Some(s) if s == "COMPLETED" => ("FINISH", "Studio Export", "Enhancement complete. Ready for download."),
            _ => ("IDLE", "Standby", "Awaiting engine handshake.")
        }
    };

    let nav_home = navigate.clone();
    let nav_history = navigate.clone();

    view! {
        <div class="editor-shell fade-in">
            <div class="editor-main-container">
                // Standardised Page Header
                <div class="page-header" style="padding: var(--s-10) var(--s-12) 0;">
                    <div class="header-main">
                        <h1 class="stagger-1 text-gradient">{TXT.editor_page_title}</h1>
                        <p class="muted stagger-2">{TXT.editor_page_subtitle}</p>
                    </div>
                </div>

                <div class="editor-main"
                     on:dragover=move |ev| { ev.prevent_default(); set_is_dragging.set(true); }
                     on:dragleave=move |_| set_is_dragging.set(false)
                     on:drop=on_drop
                >
                    <div class="editor-canvas">
                        <div class="canvas-grid"></div>
                        
                        // Workspace Header Overlay (Technical feel)
                        <div class="workspace-nav stagger-3">
                            <div class="nav-item">
                                <span class="nav-label">"WORKSPACE:"</span>
                                <span class="nav-val">"MODERN_ASSET_01"</span>
                            </div>
                            <div class="nav-divider"></div>
                            <div class="nav-item">
                                <span class="nav-label">"ENGINE:"</span>
                                <span class="nav-val accent">"ONLINE"</span>
                            </div>
                        </div>

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
                                                <div class="trigger-icon"><ImageIcon size={48} /></div>
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
                                    <h2>"DROP TO IMPORT"</h2>
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
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <label class="group-label-editor">{TXT.label_resolution}</label>
                                                <div class="tooltip-wrapper-editor">
                                                    <Info size={12} />
                                                    <div class="tooltip-editor">{TXT.desc_resolution}</div>
                                                </div>
                                            </div>
                                            <div class="resolution-grid-editor">
                                                <div 
                                                    class=move || if global_state.quality.get() == "2K" { "res-tile-editor active" } else { "res-tile-editor" }
                                                    on:click=move |_| global_state.set_quality.set("2K".to_string())
                                                >
                                                    <span class="res-num-editor">"2K"</span>
                                                    <span class="res-desc-editor">"RESTORE"</span>
                                                    <div class="res-tag-editor">"2 CREDITS"</div>
                                                </div>
                                                <div 
                                                    class=move || if global_state.quality.get() == "4K" { "res-tile-editor active" } else { "res-tile-editor" }
                                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                                >
                                                    <span class="res-num-editor">"4K"</span>
                                                    <span class="res-desc-editor">"ULTRA HD"</span>
                                                    <div class="res-tag-editor">"4 CREDITS"</div>
                                                </div>
                                            </div>
                                        </div>

                                        // Style
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <label class="group-label-editor">{TXT.label_style}</label>
                                                <div class="tooltip-wrapper-editor">
                                                    <Info size={12} />
                                                    <div class="tooltip-editor">{TXT.desc_style}</div>
                                                </div>
                                            </div>
                                            <div class="style-switcher-editor">
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

                                        // Creativity
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <div style="display: flex; gap: var(--s-2); align-items: center;">
                                                    <label class="group-label-editor">{TXT.label_creativity}</label>
                                                    <div class="tooltip-wrapper-editor">
                                                        <Info size={12} />
                                                        <div class="tooltip-editor">{TXT.desc_creativity}</div>
                                                    </div>
                                                </div>
                                                <span class="drift-val-editor">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                            </div>
                                            <div class="slider-wrapper-editor">
                                                <input 
                                                    type="range" min="0.0" max="2.0" step="0.1"
                                                    prop:value=move || global_state.temperature.get().to_string()
                                                    on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                                />
                                                <div class="slider-labels-editor">
                                                    <span>"STRICT"</span>
                                                    <span>"CREATIVE"</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Seed
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <div style="display: flex; gap: var(--s-2); align-items: center;">
                                                    <label class="group-label-editor">{TXT.label_seed}</label>
                                                    <div class="tooltip-wrapper-editor">
                                                        <Info size={12} />
                                                        <div class="tooltip-editor">{TXT.desc_seed}</div>
                                                    </div>
                                                </div>
                                                <span class="drift-val-pill-editor">
                                                    {move || global_state.seed.get().map(|s: u32| s.to_string()).unwrap_or_else(|| "AUTO".to_string())}
                                                </span>
                                            </div>
                                            <div class="seed-control-editor">
                                                <input 
                                                    type="number" 
                                                    class="studio-input-editor seed-input-editor"
                                                    placeholder="000000"
                                                    prop:value=move || global_state.seed.get().map(|s: u32| s.to_string()).unwrap_or_default()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        if val.is_empty() {
                                                            global_state.set_seed.set(None);
                                                        } else if let Ok(s) = val.parse::<u32>() {
                                                            global_state.set_seed.set(Some(s));
                                                        }
                                                    }
                                                />
                                                <div class="seed-actions-editor">
                                                    <button 
                                                        class="seed-btn-editor" 
                                                        on:click=move |_| {
                                                            let val = (js_sys::Math::random() * (u32::MAX as f64)) as u32;
                                                            global_state.set_seed.set(Some(val));
                                                        }
                                                    >
                                                        <RefreshCw size={14} />
                                                    </button>
                                                    <button 
                                                        class="seed-btn-editor"
                                                        on:click=move |_| global_state.set_seed.set(None)
                                                    >
                                                        <LogOut size={14} custom_style="transform: rotate(90deg)".to_string() />
                                                    </button>
                                                </div>
                                            </div>
                                        </div>

                                        // Advanced
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <label class="group-label-editor">{TXT.label_locks}</label>
                                                <div class="tooltip-wrapper-editor">
                                                    <Info size={12} />
                                                    <div class="tooltip-editor">{TXT.desc_locks}</div>
                                                </div>
                                            </div>
                                            <div 
                                                class="advanced-toggle-editor" 
                                                class:active=move || global_state.keep_depth_of_field.get()
                                                on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                            >
                                                <div class="toggle-icon-editor"><Target size={14} /></div>
                                                <div class="toggle-meta-editor">
                                                    <span class="toggle-title-editor">"DEPTH-OF-FIELD"</span>
                                                    <span class="toggle-sub-editor">"Preserve focal planes"</span>
                                                </div>
                                                <div class="toggle-check-editor">
                                                    <div class="check-dot-editor"></div>
                                                </div>
                                            </div>
                                        </div>

                                        // Lighting
                                        <div class="input-group-editor">
                                            <div class="label-row-editor">
                                                <label class="group-label-editor">{TXT.label_lighting}</label>
                                                <div class="tooltip-wrapper-editor">
                                                    <Info size={12} />
                                                    <div class="tooltip-editor">{TXT.desc_lighting}</div>
                                                </div>
                                            </div>
                                            <div class="studio-select-wrapper-editor">
                                                <select 
                                                    class="studio-select-editor"
                                                    on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                                    prop:value=move || global_state.lighting.get()
                                                >
                                                    <option value="Original">"ORIGINAL LIGHTING"</option>
                                                    <option value="Studio">"STUDIO LIGHTING"</option>
                                                    <option value="Cinematic">"CINEMATIC SHADOWS"</option>
                                                    <option value="Vivid">"VIVID DYNAMICS"</option>
                                                    <option value="Natural">"NATURAL OVERCAST"</option>
                                                </select>
                                                <div class="select-arrow-editor"></div>
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
                                                <span>"INITIATE"</span>
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
                                            <div class="status-icon-box">
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
                                                    <span>"RUNNING"</span>
                                                </div>
                                            </div>

                                            {move || engine_status.get().and_then(|s| s.latency_ms).map(|ms| view! {
                                                <div class="latency-telemetry fade-in">
                                                    <span class="latency-label">"DURATION:"</span>
                                                    <span class="latency-value">{format!("{:.1}s", ms as f32 / 1000.0)}</span>
                                                </div>
                                            })}
                                        </div>

                                        <div class="sidebar-note">
                                            <p>"Image is being processed by the Gemini Vision infrastructure."</p>
                                            {move || if engine_status.get().map(|s| s.status == "COMPLETED").unwrap_or(false) {
                                                let n2 = n.clone();
                                                view! {
                                                    <button class="btn btn-primary" style="margin-top: var(--s-4); width: 100%;" on:click=move |_| n2("/history", Default::default())>"VIEW GALLERY"</button>
                                                }.into_any()
                                            } else {
                                                view! { <p style="font-size: 0.75rem; opacity: 0.5;">"Do not close until completion."</p> }.into_any()
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
                background: #040404;
                overflow: hidden;
            }

            .editor-main-container {
                flex: 1;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            }

            .editor-main {
                flex: 1;
                position: relative;
                overflow: hidden;
                display: flex;
                flex-direction: column;
            }

            .editor-canvas {
                flex: 1;
                display: flex;
                align-items: center;
                justify-content: center;
                position: relative;
                padding: var(--s-12);
                background: radial-gradient(circle at center, #0a0a0a, #040404);
            }

            .workspace-nav {
                position: absolute; top: var(--s-4); left: var(--s-12); right: var(--s-12);
                display: flex; align-items: center; gap: var(--s-6); 
                padding: 10px 20px; background: rgba(0,0,0,0.4); backdrop-filter: blur(10px);
                border: 1px solid rgba(255,255,255,0.05); border-radius: 100px;
                width: fit-content; z-index: 60;
            }
            .nav-item { display: flex; align-items: center; gap: 8px; }
            .nav-label { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim) / 0.5); letter-spacing: 0.05em; }
            .nav-val { font-size: 0.625rem; font-weight: 900; color: white; letter-spacing: 0.02em; }
            .nav-val.accent { color: hsl(var(--accent)); }
            .nav-divider { width: 1px; height: 12px; background: rgba(255,255,255,0.1); }

            .canvas-grid {
                position: absolute;
                inset: 0;
                background-image: 
                    linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px),
                    linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px);
                background-size: 30px 30px;
                mask-image: radial-gradient(circle at center, black, transparent 90%);
            }

            .asset-frame {
                position: relative;
                max-width: 90%;
                max-height: 85%;
                z-index: 10;
            }

            .studio-asset {
                display: block;
                max-width: 100%;
                max-height: 60vh;
                border-radius: 2px;
                box-shadow: 0 40px 100px rgba(0,0,0,0.9), 0 0 0 1px rgba(255,255,255,0.08);
            }

            .asset-wrapper { position: relative; }

            .laser-scanner {
                position: absolute;
                top: 0; left: 0; right: 0; height: 1px;
                background: linear-gradient(90deg, transparent, hsl(var(--accent)), transparent);
                box-shadow: 0 0 20px hsl(var(--accent));
                animation: scan 5s linear infinite;
                z-index: 20;
            }

            @keyframes scan {
                0% { top: 0%; opacity: 0; }
                10%, 90% { opacity: 1; }
                100% { top: 100%; opacity: 0; }
            }

            .corner-accents .corner {
                position: absolute; width: 14px; height: 14px;
                border: 1px solid hsl(var(--accent) / 0.4);
                z-index: 25;
            }
            .corner.tl { top: -8px; left: -8px; border-right: 0; border-bottom: 0; }
            .corner.tr { top: -8px; right: -8px; border-left: 0; border-bottom: 0; }
            .corner.bl { bottom: -8px; left: -8px; border-right: 0; border-top: 0; }
            .corner.br { bottom: -8px; right: -8px; border-left: 0; border-top: 0; }

            .empty-canvas {
                text-align: center; color: hsl(var(--text-dim));
            }
            .drop-zone-trigger {
                cursor: pointer; padding: var(--s-20); border: 1px dashed rgba(255,255,255,0.08);
                border-radius: var(--radius-lg); transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                background: rgba(255,255,255,0.01);
            }
            .drop-zone-trigger:hover { 
                background: rgba(255,255,255,0.03); 
                border-color: hsl(var(--accent) / 0.4);
                transform: translateY(-4px);
            }
            .trigger-icon { 
                width: 80px; height: 80px; background: rgba(255,255,255,0.02);
                border-radius: 20px; display: flex; align-items: center; justify-content: center;
                margin: 0 auto var(--s-6); border: 1px solid rgba(255,255,255,0.05);
                color: hsl(var(--text-dim) / 0.4);
            }

            .drag-overlay {
                position: absolute; inset: -40px; background: hsla(240, 10%, 4%, 0.95);
                backdrop-filter: blur(20px); z-index: 100; border: 2px dashed hsl(var(--accent));
                border-radius: var(--radius-xl); display: flex; flex-direction: column; align-items: center; justify-content: center;
                gap: var(--s-4); color: hsl(var(--accent));
            }

            .canvas-telemetry {
                position: absolute; bottom: var(--s-6); left: 50%; transform: translateX(-50%);
                display: flex; gap: var(--s-4); z-index: 30;
            }
            .telemetry-pill {
                background: #000; border: 1px solid rgba(255,255,255,0.1);
                padding: 6px 14px; border-radius: 100px;
                display: flex; gap: 10px; font-size: 0.625rem; font-weight: 850; letter-spacing: 0.08em;
            }
            .telemetry-pill .label { color: hsl(var(--text-dim) / 0.4); }
            .telemetry-pill .value { color: white; }
            .telemetry-pill .value.accent { color: hsl(var(--accent)); }

            /* ─── SIDEBAR ─── */
            .editor-sidebar-wrapper {
                width: 420px; position: relative; border-left: 1px solid #111;
                background: #080808;
            }
            .sidebar-backdrop {
                position: absolute; inset: 0; background: radial-gradient(circle at top right, #0d0d0d, #080808);
            }
            .editor-sidebar {
                position: relative; height: 100%; display: flex; flex-direction: column; z-index: 10;
            }
            .sidebar-content { display: flex; flex-direction: column; height: 100%; }
            .sidebar-header { padding: var(--s-10) var(--s-10) var(--s-6); }
            .sidebar-title { font-size: 1.5rem; font-weight: 850; margin-top: var(--s-2); letter-spacing: -0.04em; }

            .sidebar-scrollable {
                flex: 1; overflow-y: auto; padding: 0 var(--s-10) var(--s-10);
            }
            
            .sidebar-scrollable::-webkit-scrollbar { width: 2px; }
            .sidebar-scrollable::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.05); }

            .input-group-editor { margin-bottom: var(--s-10); }
            .group-label-editor { 
                font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim) / 0.6); 
                letter-spacing: 0.15em; margin-bottom: 0px; display: flex; align-items: center; gap: var(--s-3); 
            }

            .tooltip-wrapper-editor { position: relative; display: flex; align-items: center; color: hsl(var(--text-dim) / 0.3); cursor: help; }
            .tooltip-editor { 
                position: absolute; bottom: 100%; left: 50%; transform: translateX(-50%) translateY(-10px);
                background: #111; color: hsl(var(--text-dim)); padding: 12px; border-radius: 8px; width: 220px;
                font-size: 0.6875rem; font-weight: 600; line-height: 1.5; opacity: 0; pointer-events: none;
                transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); border: 1px solid rgba(255,255,255,0.08); 
                box-shadow: 0 15px 40px rgba(0,0,0,0.8); z-index: 1000; text-align: center;
            }
            .tooltip-wrapper-editor:hover .tooltip-editor { opacity: 1; transform: translateX(-50%) translateY(-6px); }

            .resolution-grid-editor { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); }
            .res-tile-editor {
                background: #0a0a0a; border: 1px solid rgba(255,255,255,0.03);
                border-radius: var(--radius-lg); padding: var(--s-6);
                cursor: pointer; transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); position: relative; overflow: hidden;
            }
            .res-tile-editor:hover { background: #0f0f0f; border-color: rgba(255,255,255,0.1); }
            .res-tile-editor.active { background: #111; border-color: hsl(var(--accent)); box-shadow: 0 0 30px hsl(var(--accent) / 0.12); }
            .res-num-editor { display: block; font-size: 1.5rem; font-weight: 900; color: white; letter-spacing: -0.05em; }
            .res-desc-editor { display: block; font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim) / 0.5); margin-top: 2px; letter-spacing: 0.1em; }
            .res-tag-editor { 
                position: absolute; top: 0; right: 0; background: rgba(255,255,255,0.02); 
                padding: 4px 10px; font-size: 0.45rem; font-weight: 900; border-bottom-left-radius: 12px; color: hsl(var(--accent) / 0.5);
            }

            .style-switcher-editor { 
                display: flex; background: #0a0a0a; padding: 5px; border-radius: var(--radius-lg); 
                border: 1px solid rgba(255,255,255,0.03);
            }
            .style-switcher-editor button {
                flex: 1; border: none; background: transparent; color: hsl(var(--text-dim) / 0.4);
                padding: var(--s-3) 0; font-size: 0.625rem; font-weight: 850; border-radius: calc(var(--radius-lg) - 5px);
                cursor: pointer; transition: all 0.3s; letter-spacing: 0.1em;
            }
            .style-switcher-editor button.active { background: #161616; color: white; box-shadow: 0 8px 16px rgba(0,0,0,0.5); }

            .label-row-editor { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--s-5); min-height: 20px; }
            .drift-val-editor { background: #111; color: white; font-size: 0.75rem; font-weight: 900; font-family: var(--font-mono); padding: 3px 10px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.05); }
            .drift-val-pill-editor { background: hsl(var(--accent) / 0.05); color: hsl(var(--accent)); font-size: 0.75rem; font-weight: 900; font-family: var(--font-mono); padding: 3px 12px; border-radius: 100px; border: 1px solid hsl(var(--accent) / 0.1); }

            .slider-wrapper-editor { padding: var(--s-1) 0; }
            .slider-wrapper-editor input { width: 100%; height: 4px; background: #1a1a1a; border-radius: 10px; appearance: none; outline: none; }
            .slider-wrapper-editor input::-webkit-slider-thumb { 
                appearance: none; width: 16px; height: 16px; background: white; border-radius: 50%; cursor: pointer;
                box-shadow: 0 0 15px rgba(255,255,255,0.3); border: 4px solid #080808;
            }
            .slider-labels-editor { display: flex; justify-content: space-between; font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim) / 0.2); letter-spacing: 0.2em; margin-top: 8px; }

            .seed-control-editor { display: flex; gap: var(--s-2); margin-top: var(--s-2); }
            .seed-input-editor { 
                flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 0.8125rem; text-align: center; 
                background: #0a0a0a; border: 1px solid rgba(255,255,255,0.03); border-radius: var(--radius-md);
                color: white; padding: var(--s-3); transition: all 0.3s;
            }
            .seed-input-editor:focus { border-color: hsl(var(--accent) / 0.4); background: #0f0f0f; outline: none; }
            .seed-actions-editor { display: flex; gap: 6px; }
            .seed-btn-editor { 
                background: #0a0a0a; border: 1px solid rgba(255,255,255,0.03); 
                width: 44px; border-radius: var(--radius-md); color: hsl(var(--text-dim) / 0.3); 
                cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center;
            }
            .seed-btn-editor:hover { background: #111; color: white; border-color: rgba(255,255,255,0.1); }

            .advanced-toggle-editor {
                background: #0a0a0a; border: 1px solid rgba(255,255,255,0.03);
                padding: var(--s-4); border-radius: var(--radius-lg);
                display: flex; align-items: center; gap: var(--s-4); cursor: pointer; transition: all 0.3s;
            }
            .advanced-toggle-editor.active { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--accent) / 0.02); }
            .toggle-icon-editor { width: 36px; height: 36px; background: #111; border-radius: 10px; display: flex; align-items: center; justify-content: center; color: hsl(var(--text-dim) / 0.3); transition: all 0.3s; }
            .advanced-toggle-editor.active .toggle-icon-editor { background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); }
            .toggle-meta-editor { flex: 1; }
            .toggle-title-editor { display: block; font-size: 0.6875rem; font-weight: 850; color: white; letter-spacing: 0.02em; }
            .toggle-sub-editor { font-size: 0.55rem; font-weight: 700; color: hsl(var(--text-dim) / 0.4); }
            .toggle-check-editor { width: 44px; height: 22px; background: #000; border-radius: 100px; position: relative; padding: 4px; border: 1px solid rgba(255,255,255,0.05); }
            .check-dot-editor { width: 14px; height: 14px; background: #222; border-radius: 50%; transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
            .advanced-toggle-editor.active .check-dot-editor { transform: translateX(20px); background: hsl(var(--accent)); box-shadow: 0 0 10px hsl(var(--accent)); }

            .studio-select-wrapper-editor { position: relative; }
            .studio-select-editor {
                width: 100%; background: #0a0a0a; border: 1px solid rgba(255,255,255,0.03);
                padding: var(--s-3) var(--s-4); border-radius: var(--radius-md); color: white; font-size: 0.75rem;
                font-weight: 800; outline: none; appearance: none; cursor: pointer; letter-spacing: 0.05em;
            }
            .select-arrow-editor { 
                position: absolute; right: 14px; top: 50%; transform: translateY(-50%) rotate(45deg);
                width: 6px; height: 6px; border-right: 2px solid rgba(255,255,255,0.2); 
                border-bottom: 2px solid rgba(255,255,255,0.2); pointer-events: none;
            }

            .sidebar-footer { padding: var(--s-10); border-top: 1px solid #111; background: rgba(0,0,0,0.2); }
            .editor-submit-btn {
                width: 100%; height: 60px; background: hsl(var(--accent)); border: none; border-radius: var(--radius-xl);
                cursor: pointer; position: relative; overflow: hidden; transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .editor-submit-btn:disabled { opacity: 0.2; cursor: not_allowed; filter: grayscale(1); }
            .btn-inner { position: relative; z-index: 2; display: flex; align-items: center; justify-content: center; gap: var(--s-4); color: white; font-weight: 900; letter-spacing: 0.12em; font-size: 0.875rem; }
            .btn-cost { background: rgba(0,0,0,0.2); padding: 4px 10px; border-radius: 6px; font-size: 0.55rem; font-weight: 900; color: rgba(255,255,255,0.7); }
            .btn-glow { position: absolute; inset: 0; background: radial-gradient(circle at center, rgba(255,255,255,0.3), transparent 75%); opacity: 0; transition: opacity 0.3s; }
            .editor-submit-btn:hover:not(:disabled) { transform: translateY(-3px); box-shadow: 0 20px 40px hsla(var(--accent-h), var(--accent-s), 50%, 0.3); }
            .editor-submit-btn:active:not(:disabled) { transform: translateY(-1px); }

            /* ─── PROCESSING STATE ─── */
            .processing-state { background: black; }
            .processing-vitals { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: var(--s-12); }
            
            .status-icon-box { 
                width: 100px; height: 100px; background: hsl(var(--accent) / 0.05); 
                border-radius: 50%; display: flex; align-items: center; justify-content: center; 
                color: hsl(var(--accent)); margin-bottom: var(--s-12); border: 1px solid hsl(var(--accent) / 0.1);
            }

            .status-box { text-align: center; margin-bottom: var(--s-12); }
            .status-tag { display: inline-block; background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); font-size: 0.625rem; font-weight: 900; padding: 4px 14px; border-radius: 100px; margin-bottom: var(--s-3); border: 1px solid hsl(var(--accent) / 0.2); }
            .status-title { font-size: 1.5rem; font-weight: 850; margin-bottom: var(--s-2); letter-spacing: -0.04em; }
            .status-desc { font-size: 0.8125rem; color: hsl(var(--text-dim) / 0.5); max-width: 260px; line-height: 1.5; }

            .progress-container { width: 100%; max-width: 300px; margin-bottom: var(--s-10); }
            .progress-bar-rail { height: 2px; background: rgba(255,255,255,0.03); border-radius: 10px; overflow: hidden; margin-bottom: 10px; }
            .progress-bar-fill.active { height: 100%; background: hsl(var(--accent)); width: 100%; transform: translateX(-100%); animation: progress-slide-editor 2.5s infinite cubic-bezier(0.16, 1, 0.3, 1); }
            @keyframes progress-slide-editor { 
                0% { transform: translateX(-100%); }
                100% { transform: translateX(100%); }
            }
            .progress-labels { display: flex; justify-content: space-between; font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim) / 0.15); letter-spacing: 0.2em; }

            .latency-telemetry { display: flex; gap: 10px; align-items: center; padding: 12px 20px; background: #0a0a0a; border-radius: 10px; border: 1px solid rgba(255,255,255,0.03); }
            .latency-label { font-size: 0.625rem; color: hsl(var(--text-dim) / 0.3); font-weight: 850; letter-spacing: 0.05em; }
            .latency-value { font-size: 0.6875rem; color: hsl(var(--accent)); font-family: var(--font-mono); font-weight: 950; }

            .sidebar-note { padding: var(--s-10); text-align: center; font-size: 0.75rem; color: hsl(var(--text-dim) / 0.4); line-height: 1.6; }
            "
        </style>
    }
}

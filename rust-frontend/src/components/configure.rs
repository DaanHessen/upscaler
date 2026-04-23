use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle, Download, Info, ChevronRight};
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
                                    <div class="sidebar-scrollable">
                                        // Error message
                                        {move || error_msg.get().map(|msg| view! {
                                            <div class="sidebar-error animate-slide-up">
                                                <AlertCircle size={14} />
                                                <span>{msg}</span>
                                            </div>
                                        })}

                                        // Resolution Section
                                        <div class="sidebar-group-v2">
                                            <div class="card-tag-editor-v2">
                                                <Target size={10} />
                                                <span>"RESOLUTION TARGET"</span>
                                            </div>
                                            
                                            <div class="resolution-grid-v2">
                                                <div 
                                                    class=move || if global_state.quality.get() == "2K" { "res-tile-v2 active" } else { "res-tile-v2" }
                                                    on:click=move |_| global_state.set_quality.set("2K".to_string())
                                                >
                                                    <div class="res-info-v2">
                                                        <span class="res-num-v2">"2K"</span>
                                                        <span class="res-label-v2">"RESTORE"</span>
                                                    </div>
                                                    <div class="res-cost-v2">"2 CREDITS"</div>
                                                </div>
                                                <div 
                                                    class=move || if global_state.quality.get() == "4K" { "res-tile-v2 active" } else { "res-tile-v2" }
                                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                                >
                                                    <div class="res-info-v2">
                                                        <span class="res-num-v2">"4K"</span>
                                                        <span class="res-label-v2">"ULTRA HD"</span>
                                                    </div>
                                                    <div class="res-cost-v2">"4 CREDITS"</div>
                                                </div>
                                            </div>
                                        </div>

                                        // Engine Logic (Style & Creativity)
                                        <div class="sidebar-group-v2">
                                            <div class="card-tag-editor-v2">
                                                <Zap size={10} />
                                                <span>"RECONSTRUCTION ENGINE"</span>
                                            </div>

                                            <div class="control-card-v2">
                                                <label class="control-label-v2">{TXT.label_style.to_uppercase()}</label>
                                                <div class="style-switcher-v2">
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

                                            <div class="control-card-v2">
                                                <div class="label-row-v2">
                                                     <label class="control-label-v2">{TXT.label_creativity.to_uppercase()}</label>
                                                     <span class="control-val-v2">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                                </div>
                                                <div class="slider-box-v2">
                                                    <input 
                                                        type="range" min="0.0" max="2.0" step="0.1"
                                                        prop:value=move || global_state.temperature.get().to_string()
                                                        on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                                    />
                                                    <div class="slider-meta-v2">
                                                        <span>"STRICT"</span>
                                                        <span>"CREATIVE"</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // Advanced Telemetry (Seed & DOF)
                                        <div class="sidebar-group-v2">
                                            <div class="card-tag-editor-v2">
                                                <Settings size={10} />
                                                <span>"STUDIO TELEMETRY"</span>
                                            </div>

                                            <div class="control-card-v2">
                                                <div class="label-row-v2">
                                                     <label class="control-label-v2">{TXT.label_seed.to_uppercase()}</label>
                                                     <span class="seed-pill-v2">
                                                         {move || global_state.seed.get().map(|s: u32| format!("#{}", s)).unwrap_or_else(|| "AUTO".to_string())}
                                                     </span>
                                                </div>
                                                <div class="seed-input-wrapper-v2">
                                                    <input 
                                                        type="number" 
                                                        class="seed-input-v2"
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
                                                    <div class="seed-actions-v2">
                                                        <button 
                                                            class="seed-action-btn-v2" 
                                                            on:click=move |_| {
                                                                let val = (js_sys::Math::random() * (u32::MAX as f64)) as u32;
                                                                global_state.set_seed.set(Some(val));
                                                            }
                                                        >
                                                            <RefreshCw size={12} />
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>

                                            <div 
                                                class="lock-card-v2" 
                                                class:active=move || global_state.keep_depth_of_field.get()
                                                on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                            >
                                                <div class="lock-info-v2">
                                                    <span class="lock-title-v2">"DEPTH-OF-FIELD LOCK"</span>
                                                    <span class="lock-sub-v2">"Preserve focal planes"</span>
                                                </div>
                                                <div class="lock-toggle-v2">
                                                    <div class="toggle-track-v2">
                                                        <div class="toggle-thumb-v2"></div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // Environment
                                        <div class="sidebar-group-v2">
                                            <div class="card-tag-editor-v2">
                                                <Info size={10} />
                                                <span>"SCENE LIGHTING"</span>
                                            </div>
                                            <div class="studio-select-v2">
                                                <select 
                                                    on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                                    prop:value=move || global_state.lighting.get()
                                                >
                                                    <option value="Original">"ORIGINAL LIGHTING"</option>
                                                    <option value="Studio">"STUDIO LIGHTING"</option>
                                                    <option value="Cinematic">"CINEMATIC SHADOWS"</option>
                                                     <option value="Vivid">"VIVID DYNAMICS"</option>
                                                     <option value="Natural">"NATURAL OVERCAST"</option>
                                                </select>
                                                <ChevronRight size={12} custom_style="transform: rotate(90deg); position: absolute; right: 12px; pointer-events: none; opacity: 0.5;".to_string() />
                                            </div>
                                        </div>
                                    </div>

                                    <div class="sidebar-footer-v2">
                                        <button 
                                            class="initiate-btn-v2"
                                            on:click=handle_upscale
                                            disabled=move || global_state.temp_file.get().is_none()
                                        >
                                            <div class="initiate-btn-content">
                                                <Zap size={16} />
                                                <span>"INITIATE RECONSTRUCTION"</span>
                                                <span class="initiate-btn-cost">
                                                    {move || if global_state.quality.get() == "4K" { "4 CREDITS" } else { "2 CREDITS" }}
                                                </span>
                                            </div>
                                            <div class="initiate-btn-shine"></div>
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
                background: hsl(var(--bg));
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
                background: radial-gradient(circle at center, hsl(var(--accent) / 0.05), transparent);
            }

            .workspace-nav {
                position: absolute; top: var(--s-6); left: var(--s-6);
                display: flex; align-items: center; gap: var(--s-8); 
                padding: 12px 24px; background: var(--glass); backdrop-filter: blur(20px);
                border: 1px solid var(--glass-border); border-radius: var(--radius-md);
                width: auto; z-index: 60;
                box-shadow: var(--shadow-md);
            }
            .nav-item { display: flex; flex-direction: column; gap: 2px; }
            .nav-label { font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim) / 0.4); letter-spacing: 0.15em; text-transform: uppercase; }
            .nav-val { font-size: 0.6875rem; font-weight: 850; color: hsl(var(--text)); letter-spacing: 0.05em; font-family: var(--font-mono); }
            .nav-val.accent { color: hsl(var(--accent)); }
            .nav-divider { width: 1px; height: 20px; background: var(--glass-border); }

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
                position: absolute; inset: -40px; background: hsl(var(--bg) / 0.95);
                backdrop-filter: blur(20px); z-index: 100; border: 2px dashed hsl(var(--accent));
                border-radius: var(--radius-xl); display: flex; flex-direction: column; align-items: center; justify-content: center;
                gap: var(--s-4); color: hsl(var(--accent));
            }

            .canvas-telemetry {
                position: absolute; bottom: var(--s-6); left: 50%; transform: translateX(-50%);
                display: flex; gap: var(--s-4); z-index: 30;
            }
            .telemetry-pill {
                background: var(--glass); border: 1px solid var(--glass-border);
                padding: 6px 16px; border-radius: 100px;
                display: flex; gap: 10px; font-size: 0.625rem; font-weight: 900; letter-spacing: 0.05em;
                backdrop-filter: blur(10px);
            }
            .telemetry-pill .label { color: hsl(var(--text-dim) / 0.4); text-transform: uppercase; }
            .telemetry-pill .value { color: hsl(var(--text)); font-family: var(--font-mono); }
            .telemetry-pill .value.accent { color: hsl(var(--accent)); }

            /* ─── SIDEBAR ─── */
            .editor-sidebar-wrapper {
                width: 380px; 
                position: relative; 
                border-left: 1px solid hsl(var(--border) / 0.5);
                background: hsl(var(--surface));
                box-shadow: -10px 0 50px rgba(0,0,0,0.2);
            }
            .sidebar-backdrop {
                position: absolute; inset: 0; 
                background: linear-gradient(to bottom, hsl(var(--surface-raised) / 0.5), transparent);
            }
            .editor-sidebar {
                position: relative; height: 100%; display: flex; flex-direction: column; z-index: 10;
            }
            .sidebar-content { display: flex; flex-direction: column; height: 100%; }
            
            .sidebar-scrollable {
                flex: 1; overflow-y: auto; padding: var(--s-8) var(--s-8);
            }
            .sidebar-scrollable::-webkit-scrollbar { width: 2px; }
            .sidebar-scrollable::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); }

            .sidebar-group-v2 {
                background: hsl(var(--surface-raised) / 0.3);
                border: 1px solid var(--glass-border);
                border-radius: var(--radius-lg);
                padding: var(--s-6);
                margin-bottom: var(--s-6);
                transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .sidebar-group-v2:hover { 
                background: hsl(var(--surface-raised) / 0.5);
                border-color: hsl(var(--accent) / 0.2); 
            }

            .card-tag-editor-v2 {
                display: flex; align-items: center; gap: 8px;
                margin-bottom: var(--s-6);
            }
            .card-tag-editor-v2 span {
                font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim) / 0.5);
                letter-spacing: 0.1em;
            }
            .card-tag-editor-v2 svg { color: hsl(var(--accent)); }

            .resolution-grid-v2 { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); }
            .res-tile-v2 {
                background: hsl(var(--surface-raised) / 0.5); 
                border: 1px solid var(--glass-border);
                border-radius: var(--radius-md); padding: var(--s-4);
                cursor: pointer; transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                display: flex; justify-content: space-between; align-items: center;
            }
            .res-tile-v2:hover { background: hsl(var(--surface-bright)); border-color: hsl(var(--accent) / 0.3); }
            .res-tile-v2.active { 
                background: hsl(var(--accent) / 0.08); 
                border-color: hsl(var(--accent)); 
                box-shadow: 0 0 20px hsl(var(--accent) / 0.1);
            }
            .res-info-v2 { display: flex; flex-direction: column; gap: 2px; }
            .res-num-v2 { font-size: 1.125rem; font-weight: 850; color: hsl(var(--text)); font-family: var(--font-heading); }
            .res-label-v2 { font-size: 0.55rem; font-weight: 800; color: hsl(var(--text-dim) / 0.5); letter-spacing: 0.05em; text-transform: uppercase; }
            .res-cost-v2 { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--accent)); font-family: var(--font-mono); }

            .control-card-v2 { margin-bottom: var(--s-8); }
            .control-card-v2:last-child { margin-bottom: 0; }
            .control-label-v2 { font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim) / 0.6); letter-spacing: 0.05em; margin-bottom: var(--s-4); display: block; }
            
            .style-switcher-v2 { 
                display: flex; background: hsl(var(--surface-raised)); padding: 4px; border-radius: var(--radius-md); 
                border: 1px solid var(--glass-border);
            }
            .style-switcher-v2 button {
                flex: 1; border: none; background: transparent; color: hsl(var(--text-dim) / 0.6);
                padding: 10px 0; font-size: 0.625rem; font-weight: 900; border-radius: 6px;
                cursor: pointer; transition: all 0.2s; letter-spacing: 0.05em;
            }
            .style-switcher-v2 button.active { background: hsl(var(--bg)); color: hsl(var(--text)); box-shadow: var(--shadow-sm); }

            .label-row-v2 { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--s-3); }
            .control-val-v2 { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 900; color: hsl(var(--accent)); }
            
            .slider-box-v2 input { width: 100%; height: 2px; background: rgba(255,255,255,0.1); border-radius: 2px; appearance: none; outline: none; }
            .slider-box-v2 input::-webkit-slider-thumb { 
                appearance: none; width: 12px; height: 12px; background: white; border-radius: 50%; cursor: pointer;
                border: 2px solid #000; box-shadow: 0 0 10px rgba(255,255,255,0.2);
            }
            .slider-meta-v2 { display: flex; justify-content: space-between; font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim) / 0.2); margin-top: 6px; letter-spacing: 0.1em; }

            .seed-pill-v2 { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 900; color: hsl(var(--accent)); }
            .seed-input-wrapper-v2 { display: flex; gap: var(--s-2); margin-top: var(--s-1); }
            .seed-input-v2 { 
                flex: 1; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.05); 
                padding: 10px; border-radius: var(--radius-md); color: white; font-family: var(--font-mono); font-size: 0.75rem; text-align: center;
            }
            .seed-input-v2:focus { border-color: hsl(var(--accent) / 0.5); background: rgba(0,0,0,0.5); outline: none; }
            .seed-action-btn-v2 { 
                width: 40px; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);
                border-radius: var(--radius-md); color: hsl(var(--text-dim) / 0.4); cursor: pointer;
                display: flex; align-items: center; justify-content: center; transition: all 0.2s;
            }
            .seed-action-btn-v2:hover { background: rgba(255,255,255,0.1); color: white; }

            .lock-card-v2 {
                background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.03);
                padding: var(--s-4); border-radius: var(--radius-lg);
                display: flex; align-items: center; justify-content: space-between; cursor: pointer; transition: all 0.3s;
            }
            .lock-card-v2:hover { background: rgba(255,255,255,0.02); }
            .lock-card-v2.active { border-color: hsl(var(--accent) / 0.3); background: hsl(var(--accent) / 0.03); }
            .lock-info-v2 { display: flex; flex-direction: column; }
            .lock-title-v2 { font-size: 0.625rem; font-weight: 900; color: white; }
            .lock-sub-v2 { font-size: 0.5rem; font-weight: 700; color: hsl(var(--text-dim) / 0.4); }
            
            .toggle-track-v2 { width: 34px; height: 18px; background: #111; border-radius: 100px; padding: 3px; position: relative; border: 1px solid rgba(255,255,255,0.05); }
            .toggle-thumb-v2 { width: 12px; height: 12px; background: #333; border-radius: 50%; transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
            .lock-card-v2.active .toggle-thumb-v2 { transform: translateX(16px); background: hsl(var(--accent)); box-shadow: 0 0 10px hsl(var(--accent)); }

            .studio-select-v2 { position: relative; }
            .studio-select-v2 select {
                width: 100%; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.05);
                padding: 12px 16px; border-radius: var(--radius-md); color: white; font-size: 0.6875rem;
                font-weight: 900; letter-spacing: 0.05em; appearance: none; cursor: pointer; outline: none;
            }

            .sidebar-footer-v2 { padding: var(--s-8); background: hsl(var(--surface) / 0.8); border-top: 1px solid var(--glass-border); backdrop-filter: blur(10px); }
            .initiate-btn-v2 {
                width: 100%; height: 56px; background: hsl(var(--text)); border: none; border-radius: var(--radius-md);
                cursor: pointer; position: relative; overflow: hidden; transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
            }
            .initiate-btn-v2:disabled { opacity: 0.2; cursor: not-allowed; filter: grayscale(1); }
            .initiate-btn-content { position: relative; z-index: 2; display: flex; align-items: center; justify-content: center; gap: 12px; color: hsl(var(--bg)); font-weight: 900; letter-spacing: 0.1em; font-size: 0.8125rem; text-transform: uppercase; }
            .initiate-btn-cost { background: hsl(var(--bg) / 0.1); padding: 4px 10px; border-radius: 6px; font-size: 0.625rem; font-weight: 900; color: hsl(var(--bg)); font-family: var(--font-mono); }
            .initiate-btn-shine { position: absolute; inset: 0; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.1), transparent); transform: translateX(-100%); animation: shine 3s infinite; }
            @keyframes shine { 100% { transform: translateX(100%); } }
            .initiate-btn-v2:hover:not(:disabled) { background: hsl(var(--accent)); transform: translateY(-3px); box-shadow: 0 20px 40px hsl(var(--accent) / 0.2); }
            .initiate-btn-v2:hover:not(:disabled) .initiate-btn-content { color: white; }
            .initiate-btn-v2:hover:not(:disabled) .initiate-btn-cost { background: rgba(255,255,255,0.2); color: white; }

            /* ─── PROCESSING STATE ─── */
            .processing-state { background: hsl(var(--bg)); }
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

            .latency-telemetry { display: flex; gap: 10px; align-items: center; padding: 12px 20px; background: hsl(var(--surface-raised)); border-radius: 10px; border: 1px solid var(--glass-border); }
            .latency-label { font-size: 0.625rem; color: hsl(var(--text-dim) / 0.4); font-weight: 850; letter-spacing: 0.05em; }
            .latency-value { font-size: 0.6875rem; color: hsl(var(--accent)); font-family: var(--font-mono); font-weight: 950; }

            .sidebar-note { padding: var(--s-10); text-align: center; font-size: 0.75rem; color: hsl(var(--text-dim) / 0.4); line-height: 1.6; }
            "
        </style>
    }
}

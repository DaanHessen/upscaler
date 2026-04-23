use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings, PollResponse};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, RefreshCw, AlertCircle};

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();
    
    // Internal state for the upscale engine
    let (processing_job, set_processing_job) = signal(Option::<uuid::Uuid>::None);
    let (engine_status, set_engine_status) = signal(Option::<PollResponse>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Watch for job creation and start polling
    Effect::new(move |_| {
        if let Some(job_id) = processing_job.get() {
            let token = auth.session.get().map(|s| s.access_token);
            let n = navigate.clone();
            
            leptos::task::spawn_local(async move {
                let mut attempts = 0;
                loop {
                    match ApiClient::poll_job(job_id, token.as_deref()).await {
                        Ok(res) => {
                            let status = res.status.clone();
                            set_engine_status.set(Some(res));
                            
                            if status == "COMPLETED" {
                                // Once finished, we move to the final view page
                                n(&format!("/view/{}", job_id), Default::default());
                                break;
                            } else if status == "FAILED" {
                                set_error_msg.set(Some("Engine failure. Please try again.".to_string()));
                                set_processing_job.set(None);
                                break;
                            }
                        }
                        Err(e) => {
                            leptos::logging::error!("Poll error: {}", e);
                        }
                    }
                    attempts += 1;
                    if attempts > 300 { 
                        set_error_msg.set(Some("Request timed out.".to_string()));
                        set_processing_job.set(None);
                        break; 
                    }
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }
            });
        }
    });

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

    let preview_src = move || {
        if let Some(b64) = global_state.preview_base64.get() {
            format!("data:image/jpeg;base64,{}", b64)
        } else {
            global_state.temp_file.get()
                .map(|f| web_sys::Url::create_object_url_with_blob(&f).unwrap())
                .unwrap_or_default()
        }
    };

    let stage_info = move || {
        if let Some(res) = engine_status.get() {
            match res.status.as_str() {
                "PENDING" => {
                    let pos = res.queue_position.unwrap_or(1);
                    ("QUEUED", format!("Position #{}", pos), "Waiting for compute node...")
                },
                "PROCESSING" => {
                    ("ACTIVE", "Engaged".to_string(), "Reconstructing details...")
                },
                _ => ("STAGING", "Finalizing".to_string(), "Synchronizing asset...")
            }
        } else {
            ("ENGAGING", "Connecting".to_string(), "Initializing Engine...")
        }
    };

    view! {
        <div class="editor-shell animate-in">
            <div class="editor-main">
                // --- Top Canvas: Main Viewport ---
                <div class="editor-canvas">
                    <div class="canvas-background"></div>
                    <div class="viewport-content">
                        {move || {
                            if global_state.temp_file.get().is_some() {
                                view! { 
                                    <div class="img-wrapper">
                                        <img src=preview_src() class="editor-img" /> 
                                        
                                        // Processing Scan Effect
                                        {move || processing_job.get().is_some().then(|| view! {
                                            <div class="engine-scan-overlay">
                                                <div class="scan-line"></div>
                                                <div class="scan-shimmer"></div>
                                            </div>
                                        })}
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="editor-empty">
                                        <div class="empty-glow"></div>
                                        <ImageIcon size={64} />
                                        <h3>"Studio Canvas Empty"</h3>
                                        <p>"Return to the dashboard to upload an asset."</p>
                                        <button class="btn btn-secondary" on:click=move |_| navigate("/", Default::default())>"GO TO DASHBOARD"</button>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    // --- Canvas Telemetry ---
                    <div class="canvas-telemetry">
                        <div class="telemetry-pill">
                            <span class="label">"DETECTED:"</span>
                            <span class="value accent">{move || global_state.temp_classification.get().unwrap_or_else(|| "NONE".to_string())}</span>
                        </div>
                        <div class="telemetry-pill">
                            <span class="label">"ENGINE:"</span>
                            <span class="value">"STUDIO V2.2"</span>
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
                    
                    {move || match processing_job.get() {
                        None => view! {
                            // --- SETTINGS MODE ---
                            <div class="sidebar-content fade-in">
                                <div class="sidebar-header">
                                    <div class="card-tag">
                                        <Settings size={12} />
                                        <span>"UPSCALE PARAMETERS"</span>
                                    </div>
                                    <h2 class="sidebar-title">"Configuration"</h2>
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
                                        <label class="group-label">"TARGET RESOLUTION"</label>
                                        <div class="resolution-grid">
                                            <div 
                                                class=move || if global_state.quality.get() == "2K" { "res-tile active" } else { "res-tile" }
                                                on:click=move |_| global_state.set_quality.set("2K".to_string())
                                            >
                                                <span class="res-num">"2K"</span>
                                                <span class="res-desc">"HD RESTORE"</span>
                                                <div class="res-tag">"2C"</div>
                                            </div>
                                            <div 
                                                class=move || if global_state.quality.get() == "4K" { "res-tile active" } else { "res-tile" }
                                                on:click=move |_| global_state.set_quality.set("4K".to_string())
                                            >
                                                <span class="res-num">"4K"</span>
                                                <span class="res-desc">"ULTRA HD"</span>
                                                <div class="res-tag">"4C"</div>
                                            </div>
                                        </div>
                                    </div>

                                    // Style
                                    <div class="input-group">
                                        <label class="group-label">"RECONSTRUCTION STYLE"</label>
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

                                    // Temperature
                                    <div class="input-group">
                                        <div class="label-row">
                                            <label class="group-label">"CREATIVE DRIFT"</label>
                                            <span class="drift-val">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                        </div>
                                        <div class="slider-wrapper">
                                            <input 
                                                type="range" min="0.0" max="2.0" step="0.1"
                                                prop:value=move || global_state.temperature.get().to_string()
                                                on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                            />
                                            <div class="slider-labels">
                                                <span>"FAITHFUL"</span>
                                                <span>"CREATIVE"</span>
                                            </div>
                                        </div>
                                    </div>

                                    // Advanced
                                    <div class="input-group">
                                        <label class="group-label">"ADVANCED ENGINE LOCKS"</label>
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
                                        <label class="group-label">"ATMOSPHERIC LIGHTING"</label>
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
                                                {move || if global_state.quality.get() == "4K" { "4C" } else { "2C" }}
                                            </div>
                                        </div>
                                        <div class="btn-glow"></div>
                                    </button>
                                </div>
                            </div>
                        }.into_any(),

                        Some(_) => view! {
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
                                </div>

                                <div class="sidebar-note">
                                    <p>"Your image is being processed by the Gemini Vision infrastructure. Do not close this tab until completion."</p>
                                </div>
                            </div>
                        }.into_any()
                    }}
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
                padding: var(--s-12);
            }

            .canvas-background {
                position: absolute;
                inset: 0;
                background: radial-gradient(circle at center, #111 0%, #050505 100%);
                opacity: 0.8;
                z-index: 1;
            }

            .viewport-content {
                position: relative;
                z-index: 2;
                max-width: 100%;
                max-height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
            }

            /* ─── IMAGE WRAPPER ─── */
            .img-wrapper {
                position: relative;
                border-radius: var(--radius-sm);
                overflow: hidden;
                box-shadow: 0 40px 100px -30px rgba(0,0,0,0.8), 0 0 0 1px rgba(255,255,255,0.05);
            }

            .editor-img {
                max-width: 100%;
                max-height: calc(100vh - 200px);
                display: block;
                object-fit: contain;
            }

            /* ─── SCAN EFFECT ─── */
            .engine-scan-overlay {
                position: absolute;
                inset: 0;
                pointer-events: none;
            }

            .scan-line {
                position: absolute;
                left: 0;
                width: 100%;
                height: 2px;
                background: hsl(var(--accent));
                box-shadow: 0 0 20px hsl(var(--accent)), 0 0 40px hsl(var(--accent) / 0.5);
                animation: scanner-loop 3s cubic-bezier(0.4, 0, 0.2, 1) infinite;
            }

            @keyframes scanner-loop {
                0% { top: 0%; opacity: 0; }
                10% { opacity: 1; }
                90% { opacity: 1; }
                100% { top: 100%; opacity: 0; }
            }

            .scan-shimmer {
                position: absolute;
                inset: 0;
                background: linear-gradient(to bottom, transparent, hsl(var(--accent) / 0.1), transparent);
                height: 100px;
                animation: shimmer-loop 3s cubic-bezier(0.4, 0, 0.2, 1) infinite;
            }

            @keyframes shimmer-loop {
                0% { top: -100px; }
                100% { top: 100%; }
            }

            /* ─── TELEMETRY ─── */
            .canvas-telemetry {
                position: absolute;
                bottom: var(--s-8);
                left: 50%;
                transform: translateX(-50%);
                display: flex;
                gap: var(--s-4);
                z-index: 5;
            }

            .telemetry-pill {
                background: rgba(20,20,20,0.7);
                backdrop-filter: blur(10px);
                border: 1px solid rgba(255,255,255,0.05);
                padding: var(--s-2) var(--s-4);
                border-radius: 100px;
                display: flex;
                gap: var(--s-2);
                font-size: 0.625rem;
                white-space: nowrap;
            }

            .telemetry-pill .label { color: hsl(var(--text-dim)); font-weight: 800; }
            .telemetry-pill .value { color: hsl(var(--text)); font-weight: 700; }
            .telemetry-pill .value.accent { color: hsl(var(--accent)); }
            .job-pill { border-color: hsl(var(--accent) / 0.3); }

            /* ─── SIDEBAR ─── */
            .editor-sidebar-wrapper {
                width: 380px;
                position: relative;
                border-left: 1px solid rgba(255,255,255,0.05);
                z-index: 10;
            }

            .sidebar-backdrop {
                position: absolute;
                inset: 0;
                background: #0a0a0a;
                opacity: 0.8;
            }

            .editor-sidebar {
                position: relative;
                height: 100%;
                display: flex;
                flex-direction: column;
                padding: var(--s-8);
                color: hsl(var(--text));
            }

            .sidebar-content {
                display: flex;
                flex-direction: column;
                height: 100%;
            }

            .sidebar-header { margin-bottom: var(--s-10); }
            .sidebar-title { font-size: 1.75rem; font-weight: 850; letter-spacing: -0.04em; margin-top: var(--s-2); }

            .sidebar-scrollable {
                flex: 1;
                overflow-y: auto;
                padding-right: var(--s-2);
                margin-right: calc(-1 * var(--s-2));
            }

            .sidebar-scrollable::-webkit-scrollbar { width: 4px; }
            .sidebar-scrollable::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 10px; }

            .input-group { margin-bottom: var(--s-8); }
            .group-label { font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; margin-bottom: var(--s-3); display: block; }

            /* ─── TILES ─── */
            .resolution-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); }
            .res-tile {
                background: #151515;
                border: 1px solid rgba(255,255,255,0.05);
                border-radius: var(--radius-md);
                padding: var(--s-6);
                cursor: pointer;
                transition: all 0.2s;
                position: relative;
                overflow: hidden;
            }

            .res-tile:hover { border-color: rgba(255,255,255,0.15); background: #1a1a1a; }
            .res-tile.active { border-color: hsl(var(--accent)); background: hsl(var(--accent) / 0.1); }
            .res-num { font-size: 1.5rem; font-weight: 800; display: block; line-height: 1; margin-bottom: 2px; }
            .res-desc { font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 700; text-transform: uppercase; }
            .res-tag { position: absolute; top: var(--s-3); right: var(--s-4); font-size: 0.625rem; font-weight: 950; opacity: 0.3; }
            .res-tile.active .res-tag { opacity: 1; color: hsl(var(--accent)); }

            /* ─── SWITCHER ─── */
            .style-switcher { 
                display: flex; gap: 2px; background: #151515; padding: 2px; border-radius: var(--radius-md); 
                border: 1px solid rgba(255,255,255,0.05);
            }
            .style-switcher button {
                flex: 1; border: none; background: transparent; color: hsl(var(--text-dim));
                padding: var(--s-3) 0; font-size: 0.6875rem; font-weight: 800; border-radius: calc(var(--radius-md) - 2px);
                cursor: pointer; transition: all 0.2s;
            }
            .style-switcher button.active { background: #252525; color: hsl(var(--text)); font-weight: 900; }

            /* ─── TOGGLE ─── */
            .advanced-toggle {
                background: #151515; border: 1px solid rgba(255,255,255,0.05); border-radius: var(--radius-md);
                padding: var(--s-4); display: flex; align-items: center; gap: var(--s-4); cursor: pointer; transition: all 0.2s;
            }
            .advanced-toggle.active { border-color: hsl(var(--accent) / 0.5); background: hsl(var(--accent) / 0.05); }
            .toggle-icon { width: 32px; height: 32px; border-radius: 8px; background: rgba(255,255,255,0.03); display: flex; align-items: center; justify-content: center; color: hsl(var(--text-dim)); }
            .advanced-toggle.active .toggle-icon { color: hsl(var(--accent)); background: hsl(var(--accent) / 0.1); }
            .toggle-meta { flex: 1; display: flex; flex-direction: column; }
            .toggle-title { font-size: 0.6875rem; font-weight: 850; letter-spacing: 0.02em; }
            .toggle-sub { font-size: 0.625rem; color: hsl(var(--text-dim)); }
            .toggle-check { width: 32px; height: 16px; background: #252525; border-radius: 100px; position: relative; }
            .check-dot { position: absolute; left: 3px; top: 3px; width: 10px; height: 10px; background: #555; border-radius: 50%; transition: all 0.2s; }
            .advanced-toggle.active .check-dot { left: calc(100% - 13px); background: hsl(var(--accent)); }

            /* ─── SELECT ─── */
            .studio-select {
                width: 100%; background: #151515; border: 1px solid rgba(255,255,255,0.05);
                border-radius: var(--radius-md); padding: var(--s-4); color: white;
                font-size: 0.75rem; font-weight: 800; appearance: none; cursor: pointer;
            }

            /* ─── SUBMIT BTN ─── */
            .sidebar-footer { padding-top: var(--s-8); }
            .editor-submit-btn {
                width: 100%; background: transparent; border: none; padding: 0; cursor: pointer;
                position: relative; overflow: hidden; border-radius: var(--radius-lg);
            }
            .btn-inner {
                background: linear-gradient(135deg, hsl(var(--accent)) 0%, #6366f1 100%);
                padding: var(--s-6); display: flex; align-items: center; justify-content: center; gap: var(--s-3);
                color: white; position: relative; z-index: 2; border-radius: var(--radius-lg);
            }
            .btn-inner span { font-family: var(--font-heading); font-weight: 900; font-size: 1rem; letter-spacing: 0.05em; }
            .btn-cost { background: rgba(0,0,0,0.2); padding: 4px 10px; border-radius: 100px; font-weight: 950; font-size: 0.625rem; }
            .btn-glow { position: absolute; inset: -20px; background: radial-gradient(circle, hsl(var(--accent) / 0.5) 0%, transparent 70%); opacity: 0; transition: opacity 0.3s; z-index: 1; }
            .editor-submit-btn:hover .btn-glow { opacity: 1; }
            .editor-submit-btn:active { transform: scale(0.98); }

            /* ─── PROCESSING VITALS ─── */
            .processing-state { items-align: center; text-align: center; padding-top: var(--s-10); }
            .pulse-ring { position: relative; width: 120px; height: 120px; margin: 0 auto var(--s-10); display: flex; align-items: center; justify-content: center; color: hsl(var(--accent)); }
            .ring { position: absolute; border: 2px solid hsl(var(--accent)); border-radius: 50%; animation: ring-pulse 2s infinite; opacity: 0; }
            .ring.r1 { animation-delay: 0s; }
            .ring.r2 { animation-delay: 1s; }
            @keyframes ring-pulse {
                0% { width: 40px; height: 40px; opacity: 0.6; border-width: 4px; }
                100% { width: 140px; height: 140px; opacity: 0; border-width: 1px; }
            }

            .status-box { margin-bottom: var(--s-10); }
            .status-tag { font-size: 0.625rem; font-weight: 950; color: hsl(var(--accent)); letter-spacing: 0.2em; display: block; margin-bottom: 4px; }
            .status-title { font-size: 1.5rem; font-weight: 900; margin-bottom: var(--s-2); }
            .status-desc { font-size: 0.8125rem; color: hsl(var(--text-dim)); line-height: 1.5; }

            .progress-container { width: 100%; margin-top: var(--s-10); }
            .progress-bar-rail { width: 100%; height: 6px; background: #151515; border-radius: 100px; overflow: hidden; margin-bottom: var(--s-3); }
            .progress-bar-fill.active { 
                width: 30%; height: 100%; background: hsl(var(--accent)); border-radius: 100px; 
                animation: flow-load 2s linear infinite; 
            }
            @keyframes flow-load {
                0% { transform: translateX(-100%); }
                100% { transform: translateX(400%); }
            }
            .progress-labels { display: flex; justify-content: space-between; font-size: 0.5625rem; font-weight: 900; color: hsl(var(--text-dim)); opacity: 0.5; }

            .sidebar-error { background: hsl(var(--error) / 0.1); border: 1px solid hsl(var(--error) / 0.2); padding: var(--s-4); border-radius: var(--radius-md); color: hsl(var(--error)); display: flex; gap: var(--s-3); align-items: center; font-size: 0.6875rem; font-weight: 700; margin-bottom: var(--s-6); }

            /* ─── RESPONSIVE ─── */
            @media (max-width: 1000px) {
                .editor-shell { flex-direction: column; height: auto; }
                .editor-sidebar-wrapper { width: 100%; border-left: none; border-top: 1px solid rgba(255,255,255,0.05); }
                .editor-img { max-height: 50vh; }
            }
            "
        </style>
    }
}

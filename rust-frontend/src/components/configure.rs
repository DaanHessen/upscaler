use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings};
use crate::components::icons::{Zap, ImageIcon, Settings, Maximize, Target, Sun};

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();
    
    let (loading, set_loading) = signal(false);

    // Classification should only update STYLE if the user hasn't manually tweaked it yet
    // Or we just let it override for now as per "AI auto-detection preference"
    Effect::new(move |_| {
        if let Some(cls) = global_state.temp_classification.get() {
            global_state.set_style.set(cls);
        }
    });

    let handle_upscale = move |_| {
        let navigate = navigate.clone();
        if let Some(file) = global_state.temp_file.get() {
            let q_val: String = global_state.quality.get();
            let cost = if q_val == "4K" { 4 } else { 2 };
            
            // Check credits
            if let Some(current) = auth.credits.get() {
                if current < cost {
                    leptos::logging::error!("Insufficient credits");
                    return;
                }
            }

            set_loading.set(true);
            let token = auth.session.get().map(|s| s.access_token);
            let s_val: String = global_state.style.get();
            let t_val: f32 = global_state.temperature.get();
            let auth_ctx = auth;
            
            let p_settings = PromptSettings {
                keep_aspect_ratio: global_state.keep_aspect_ratio.get(),
                keep_depth_of_field: global_state.keep_depth_of_field.get(),
                lighting: global_state.lighting.get(),
                thinking_level: global_state.thinking_level.get(),
            };
            
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, &p_settings, token.as_deref()).await {
                    Ok(resp) => {
                        // Optimistic update
                        auth_ctx.set_credits.update(|c| if let Some(cv) = c { *cv -= cost; });
                        auth_ctx.sync_telemetry(true);
                        navigate(&format!("/view/{}", resp.job_id), Default::default());
                    },
                    Err(e) => {
                        leptos::logging::error!("Upscale failed: {}", e);
                        set_loading.set(false);
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

    view! {
        <div class="configure-page fade-in">
            <div class="page-header stagger-1">
                <div class="header-main">
                    <h1 class="text-gradient">"Upscale Settings"</h1>
                    <p class="muted">"Precision restoration parameters for your assets."</p>
                </div>
            </div>

            <div class="config-layout stagger-2">
                <div class="config-stage shadow-xl">
                    <div class="preview-section">
                        <div class="section-tag">
                            <ImageIcon size={10} />
                            "ASSET PREVIEW"
                        </div>
                        <div class="preview-viewport">
                            {move || {
                                let has_file = global_state.temp_file.get().is_some();
                                if has_file {
                                    view! { <img src=preview_src() class="fade-in" /> }.into_any()
                                } else {
                                    view! { <div class="empty-state">"No asset detected"</div> }.into_any()
                                }
                            }}
                            <div class="viewport-overlay">
                                <div class="badge">
                                    <span class="label">"Status"</span>
                                    <div class="badge-content">
                                        <div class="pulse-dot"></div>
                                        {move || global_state.temp_classification.get().unwrap_or_else(|| "Analyzing...".to_string())}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="controls-section">
                        <div class="section-tag">
                            <Settings size={10} />
                            "CONFIGURATION"
                        </div>

                        <div class="controls-grid">
                            <div class="control-column">
                                <div class="control-group">
                                    <label>"Target Resolution"</label>
                                    <div class="resolution-switch">
                                        <div 
                                            class=move || if global_state.quality.get() == "2K" { "res-opt active" } else { "res-opt" }
                                            on:click=move |_| global_state.set_quality.set("2K".to_string())
                                        >
                                            <span class="res-title">"2K"</span>
                                            <span class="res-sub">"HD RESTORE"</span>
                                        </div>
                                        <div 
                                            class=move || if global_state.quality.get() == "4K" { "res-opt active" } else { "res-opt" }
                                            on:click=move |_| global_state.set_quality.set("4K".to_string())
                                        >
                                            <span class="res-title">"4K"</span>
                                            <span class="res-sub">"ULTRA HD"</span>
                                        </div>
                                    </div>
                                </div>

                                <div class="control-group">
                                    <label>"Reconstruction Style"</label>
                                    <div class="style-toggle">
                                        <button 
                                            class=move || if global_state.style.get() == "PHOTOGRAPHY" { "toggle-btn active" } else { "toggle-btn" }
                                            on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                        >
                                            <Sun size={12} />
                                            "PHOTOGRAPHY"
                                        </button>
                                        <button 
                                            class=move || if global_state.style.get() == "ILLUSTRATION" { "toggle-btn active" } else { "toggle-btn" }
                                            on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                        >
                                            <ImageIcon size={12} />
                                            "ILLUSTRATION"
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <div class="control-column">
                                <div class="control-group">
                                    <label>"Neural Temperature"</label>
                                    <div class="slider-container">
                                        <div class="slider-header">
                                            <span>"Creative Drift"</span>
                                            <span class="val-pill">{move || format!("{:.1}", global_state.temperature.get())}</span>
                                        </div>
                                        <input 
                                            type="range" 
                                            min="0.0" 
                                            max="2.0" 
                                            step="0.1" 
                                            prop:value=move || global_state.temperature.get().to_string()
                                            on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                        />
                                    </div>
                                </div>

                                <div class="control-group">
                                    <label>"Advanced Preserves"</label>
                                    <div class="toggles-row">
                                        <button 
                                            class=move || if global_state.keep_aspect_ratio.get() { "pill-toggle active" } else { "pill-toggle" }
                                            on:click=move |_| global_state.set_keep_aspect_ratio.update(|v| *v = !*v)
                                        >
                                            <Maximize size={12} />
                                            "Ratio Lock"
                                        </button>
                                        <button 
                                            class=move || if global_state.keep_depth_of_field.get() { "pill-toggle active" } else { "pill-toggle" }
                                            on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                        >
                                            <Target size={12} />
                                            "Depth Lock"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="lighting-group">
                            <label>"Atmospheric Lighting"</label>
                            <div class="select-box">
                                <select 
                                    on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                    prop:value=move || global_state.lighting.get()
                                >
                                    <option value="Original">"Maintain Original (Default)"</option>
                                    <option value="Studio">"Studio Lighting"</option>
                                    <option value="Cinematic">"Cinematic Shadowing"</option>
                                    <option value="Vivid">"High Vividity"</option>
                                    <option value="Natural">"Soft Ambient"</option>
                                </select>
                            </div>
                        </div>

                        <div class="action-footer">
                            <button 
                                class="btn-upscale"
                                disabled=move || loading.get() || global_state.temp_file.get().is_none()
                                on:click=handle_upscale
                            >
                                <div class="btn-ripple"></div>
                                <Zap size={18} />
                                <span>{move || if loading.get() { "STARTING ENGINE..." } else { "INITIATE UPSCALE" }}</span>
                                <div class="cost-tag">
                                    {move || {
                                        let q = global_state.quality.get();
                                        if q == "4K" { "4 CREDITS" } else { "2 CREDITS" }
                                    }}
                                </div>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".configure-page { max-width: 1200px; margin: 0 auto; padding: 0 var(--s-8) var(--s-8) var(--s-8); min-height: 80vh; display: flex; flex-direction: column; }
                .page-header { margin-bottom: 3rem; }
                .page-header h1 { font-size: 2.5rem; font-weight: 900; letter-spacing: -0.05em; margin-bottom: 0.5rem; }
                .page-header .muted { font-size: 1rem; color: hsl(var(--text-muted)); font-weight: 500; }
                
                .config-layout { flex: 1; display: flex; flex-direction: column; }
                .config-stage { 
                    background: hsl(var(--surface)); 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-lg); 
                    overflow: hidden;
                    display: grid;
                    grid-template-columns: 420px 1fr;
                    flex: 1;
                    min-height: 650px;
                }

                .section-tag { font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.15em; margin-bottom: 1.5rem; opacity: 0.5; display: flex; align-items: center; gap: 0.5rem; }

                /* Left Side: Preview */
                .preview-section { 
                    background: hsl(var(--surface-raised) / 0.3);
                    border-right: 1px solid var(--glass-border);
                    padding: 2rem;
                    display: flex;
                    flex-direction: column;
                }
                .preview-viewport { 
                    flex: 1;
                    background: #000;
                    border-radius: var(--radius-md);
                    border: 1px solid var(--glass-border);
                    position: relative;
                    overflow: hidden;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                .preview-viewport img { width: 100%; height: 100%; object-fit: contain; }
                .empty-state { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-dim)); opacity: 0.3; }

                .viewport-overlay { position: absolute; bottom: 1.5rem; left: 1.5rem; right: 1.5rem; }
                .badge { 
                    background: hsl(var(--bg) / 0.85); 
                    backdrop-filter: blur(12px); 
                    border: 1px solid var(--glass-border);
                    padding: 0.625rem 1rem;
                    border-radius: var(--radius-md);
                    display: flex;
                    flex-direction: column;
                    gap: 4px;
                    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
                }
                .badge .label { font-size: 0.55rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; }
                .badge-content { display: flex; align-items: center; gap: var(--s-3); font-size: 0.8125rem; font-weight: 800; color: hsl(var(--accent)); text-transform: uppercase; letter-spacing: 0.05em; }
                .pulse-dot { width: 8px; height: 8px; background: hsl(var(--accent)); border-radius: 50%; box-shadow: 0 0 12px hsl(var(--accent)); animation: pulse-accent 2s infinite; }

                /* Right Side: Controls */
                .controls-section { padding: 3rem; display: flex; flex-direction: column; gap: 2.5rem; }
                .controls-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 3rem; }
                .control-group { display: flex; flex-direction: column; gap: 1.25rem; }
                .control-group label { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.15em; }

                /* Resolution Switch */
                .resolution-switch { display: grid; grid-template-columns: 1fr 1fr; background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 5px; gap: 5px; }
                .res-opt { padding: 1rem; border-radius: var(--radius-sm); display: flex; flex-direction: column; align-items: center; cursor: pointer; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1); border: 1px solid transparent; }
                .res-opt.active { background: hsl(var(--accent)); border-color: hsl(var(--accent)); transform: translateY(-2px); box-shadow: 0 8px 24px hsl(var(--accent) / 0.3); }
                .res-opt.active .res-title { color: white; }
                .res-opt.active .res-sub { color: rgba(255, 255, 255, 0.7); }
                .res-title { font-size: 1.25rem; font-weight: 950; color: hsl(var(--text)); letter-spacing: -0.02em; }
                .res-sub { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.05rem; }
                .res-opt:hover:not(.active) { background: hsl(var(--surface-raised)); }

                /* Style Toggle */
                .style-toggle { display: flex; flex-direction: column; gap: 0.75rem; }
                .toggle-btn { background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 1rem; font-size: 0.75rem; font-weight: 850; color: hsl(var(--text-muted)); cursor: pointer; display: flex; align-items: center; gap: 1rem; transition: all 0.2s; }
                .toggle-btn:hover { border-color: hsl(var(--accent) / 0.5); color: hsl(var(--text)); }
                .toggle-btn.active { border-color: hsl(var(--accent)); color: hsl(var(--accent)); background: hsl(var(--accent) / 0.08); box-shadow: inset 0 0 12px hsl(var(--accent) / 0.05); }

                /* Slider */
                .slider-container { background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 1.25rem; }
                .slider-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; font-size: 0.75rem; font-weight: 800; color: hsl(var(--text-muted)); letter-spacing: 0.02em; }
                .val-pill { background: hsl(var(--accent) / 0.1); color: hsl(var(--accent)); padding: 4px 10px; border-radius: 100px; font-family: var(--font-mono); font-weight: 900; font-size: 0.8125rem; border: 1px solid hsl(var(--accent) / 0.2); }

                /* Pill Toggles */
                .toggles-row { display: flex; flex-direction: column; gap: 0.75rem; }
                .pill-toggle { background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 0.875rem 1.25rem; font-size: 0.75rem; font-weight: 850; color: hsl(var(--text-muted)); cursor: pointer; display: flex; align-items: center; gap: 0.75rem; transition: all 0.2s; }
                .pill-toggle.active { border-color: hsl(var(--accent)); color: hsl(var(--accent)); background: hsl(var(--accent) / 0.08); }
                .pill-toggle:hover:not(.active) { border-color: hsl(var(--text-dim) / 0.4); }

                /* Lighting */
                .lighting-group { margin-top: 1rem; display: flex; flex-direction: column; gap: 1.25rem; }
                .lighting-group label { font-size: 0.6875rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.15em; }
                .select-box select { width: 100%; background: hsl(var(--bg)); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 1.125rem 1.25rem; color: hsl(var(--text)); font-size: 0.875rem; font-weight: 700; appearance: none; cursor: pointer; transition: all 0.2s; }
                .select-box select:hover { border-color: hsl(var(--accent) / 0.5); }
                
                /* Action Button */
                .action-footer { margin-top: auto; padding-top: 2rem; }
                .btn-upscale { 
                    width: 100%; 
                    background: linear-gradient(135deg, hsl(var(--accent)), #6366f1);
                    border: none;
                    border-radius: var(--radius-md);
                    padding: 1.5rem;
                    color: white;
                    font-size: 1.125rem;
                    font-weight: 950;
                    letter-spacing: 0.08em;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 1.25rem;
                    cursor: pointer;
                    position: relative;
                    overflow: hidden;
                    box-shadow: 0 12px 32px -12px hsl(var(--accent) / 0.6);
                    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                    text-transform: uppercase;
                }
                .btn-upscale:hover { transform: translateY(-4px); box-shadow: 0 24px 48px -15px hsl(var(--accent) / 0.7); }
                .btn-upscale:active { transform: translateY(0) scale(0.98); }
                .btn-upscale:disabled { opacity: 0.5; transform: none; filter: grayscale(1); cursor: not-allowed; }

                .cost-tag { background: rgba(0,0,0,0.25); padding: 5px 12px; border-radius: 100px; font-size: 0.6875rem; font-weight: 950; color: white; letter-spacing: 0.05em; border: 1px solid rgba(255,255,255,0.1); }

                @media (max-width: 1100px) {
                    .config-stage { grid-template-columns: 1fr; min-height: auto; }
                    .preview-section { border-right: none; border-bottom: 1px solid var(--glass-border); padding: 2rem; }
                    .preview-viewport { height: 400px; }
                    .controls-section { padding: 2rem; }
                    .controls-grid { grid-template-columns: 1fr; gap: 2rem; }
                }
                "
            </style>
        </div>
    }
}

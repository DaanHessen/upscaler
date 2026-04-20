use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::ApiClient;
use crate::components::icons::{Zap, ImageIcon, Settings};

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
            let q_val = global_state.quality.get();
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
            let s_val = global_state.style.get();
            let t_val = global_state.temperature.get();
            let auth_ctx = auth;
            
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, token.as_deref()).await {
                    Ok(resp) => {
                        // Optimistic update
                        auth_ctx.set_credits.update(|c| if let Some(cv) = c { *cv -= cost; });
                        
                        // Force refresh telemetry in background
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

    let preview_url = move || {
        global_state.temp_file.get().map(|f| web_sys::Url::create_object_url_with_blob(&f).unwrap()).unwrap_or_default()
    };

    view! {
        <div class="configure-page fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="text-gradient">"Upscale Settings"</h1>
                    <p class="muted">"Customize the reconstruction parameters for your asset."</p>
                </div>
            </div>

            <div class="config-layout">
                <div class="config-left">
                    <div class="card preview-card shadow-lg">
                        <div class="params-body">
                            <div class="card-tag">
                                <ImageIcon size={10} />
                                <span>"SOURCE ASSET"</span>
                            </div>
                            <div class="preview-visual">
                                {move || if global_state.temp_file.get().is_some() {
                                    view! { <img src=preview_url /> }.into_any()
                                } else {
                                    view! { <div class="empty-preview">"No image selected"</div> }.into_any()
                                }}
                            </div>
                            <div class="meta-stats" style="margin-top: auto; border: none; padding-top: var(--s-4);">
                                <div class="stat-box">
                                    <span class="stat-label">"Classification"</span>
                                    <div class="classification-active">
                                        <div class="scanning-icon">
                                            <Zap size={10} />
                                        </div>
                                        <span class="stat-value highlight">{move || global_state.temp_classification.get().unwrap_or("PENDING...".to_string())}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="config-right">
                    <div class="card params-card shadow-lg">
                        <div class="params-body">
                            <div class="card-tag">
                                <Settings size={10} />
                                <span>"PARAMETERS"</span>
                            </div>
                            
                            <div class="params-content">
                                <div class="param-group" title="Higher resolution requires more processing power and credits.">
                                    <label>"Target Resolution"</label>
                                    <div class="radio-group">
                                        <div 
                                            class=move || if global_state.quality.get() == "2K" { "radio-item active" } else { "radio-item" }
                                            on:click=move |_| global_state.set_quality.set("2K".to_string())
                                        >
                                            <div class="pack-info">
                                                <span class="pack-name">"2K (HD)"</span>
                                                <span class="pack-credits">"2 CREDITS"</span>
                                            </div>
                                        </div>
                                        <div 
                                            class=move || if global_state.quality.get() == "4K" { "radio-item active" } else { "radio-item" }
                                            on:click=move |_| global_state.set_quality.set("4K".to_string())
                                        >
                                            <div class="pack-info">
                                                <span class="pack-name">"4K (UHD)"</span>
                                                <span class="pack-credits">"4 CREDITS"</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="param-group">
                                    <label>"Image Type"</label>
                                    <div class="segmented-control">
                                        <button 
                                            class=move || if global_state.style.get() == "PHOTOGRAPHY" { "segment active" } else { "segment" }
                                            on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                        >
                                            "PHOTOGRAPHY"
                                        </button>
                                        <button 
                                            class=move || if global_state.style.get() == "ILLUSTRATION" { "segment active" } else { "segment" }
                                            on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                        >
                                            "ILLUSTRATION"
                                        </button>
                                    </div>
                                </div>

                                <div class="param-group">
                                    <label>"Creativity: " {move || format!("{:.1}", global_state.temperature.get())}</label>
                                    <div class="slider-wrapper">
                                        <input 
                                            type="range" 
                                            min="0.0" 
                                            max="2.0" 
                                            step="0.1" 
                                            prop:value=move || global_state.temperature.get().to_string()
                                            on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                                        />
                                        <div class="range-labels">
                                            <span>"Faithful"</span>
                                            <span>"Artistic"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="card-actions-row" style="margin-top: auto;">
                                <button 
                                    class="btn btn-primary btn-lg btn-block" 
                                    disabled=move || loading.get() || global_state.temp_file.get().is_none()
                                    on:click=handle_upscale
                                >
                                    {move || if loading.get() { "UPSCALING..." } else { "UPSCALE" }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".configure-page { width: 100%; max-width: 1200px; margin: 0 auto; }
                .page-header { margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); }
                
                .config-layout { display: grid; grid-template-columns: 1.1fr 0.9fr; gap: var(--s-12); margin-top: var(--s-6); align-items: stretch; }
                
                /* Card Geometry */
                .params-body { padding: var(--s-10); height: 100%; display: flex; flex-direction: column; }
                .card-tag { display: flex; align-items: center; gap: var(--s-2); font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.1em; margin-bottom: var(--s-8); opacity: 0.6; }
                
                .preview-card, .params-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); transition: border-color 0.3s; }
                .preview-card:hover, .params-card:hover { border-color: hsl(var(--accent) / 0.2); }

                /* Preview Section */
                .preview-visual { 
                    background: #000; 
                    display: flex; 
                    align-items: center; 
                    justify-content: center; 
                    min-height: 480px; 
                    border-radius: var(--radius-md); 
                    overflow: hidden; 
                    border: 1px solid var(--glass-border); 
                    position: relative;
                }
                .preview-visual img { max_width: 100%; max-height: 100%; object-fit: contain; }
                .empty-preview { font-size: 0.625rem; font-weight: 800; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.5; }

                /* Settings Panel */
                .params-content { display: flex; flex-direction: column; gap: var(--s-12); flex: 1; }
                .param-group label { display: block; font-size: 0.625rem; font-weight: 900; text-transform: uppercase; letter-spacing: 0.15em; color: hsl(var(--text-dim)); margin-bottom: var(--s-6); opacity: 0.8; }
                
                .radio-group { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-4); }
                .radio-item { 
                    padding: var(--s-6); 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-md); 
                    cursor: pointer; 
                    display: flex; 
                    justify-content: space-between; 
                    align-items: center; 
                    transition: all 0.2s;
                }
                .radio-item:hover { border-color: hsl(var(--accent) / 0.4); background: hsl(var(--surface-raised) / 0.4); }
                .radio-item.active { border-color: hsl(var(--accent)); background: hsl(var(--accent) / 0.05); }
                
                .pack-info { display: flex; flex-direction: column; gap: 4px; }
                .pack-name { font-size: 0.875rem; font-weight: 750; color: hsl(var(--text)); }
                .pack-credits { font-size: 0.625rem; font-weight: 850; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.05em; }

                .segmented-control { display: grid; grid-template-columns: 1fr 1fr; background: hsl(var(--surface-raised) / 0.5); border: 1px solid var(--glass-border); border-radius: var(--radius-md); padding: 4px; gap: 4px; }
                .segment { background: transparent; border: none; padding: 12px; border-radius: 4px; color: hsl(var(--text-dim)); font-size: 0.65rem; font-weight: 800; cursor: pointer; transition: all 0.2s; letter-spacing: 0.05em; }
                .segment:hover { color: hsl(var(--text)); background: hsl(var(--surface-raised)); }
                .segment.active { background: hsl(var(--accent)); color: hsl(var(--bg)); box-shadow: 0 4px 12px hsl(var(--accent) / 0.2); }
                
                .slider-wrapper { display: flex; flex-direction: column; gap: 0.75rem; }
                input[type='range'] { -webkit-appearance: none; width: 100%; background: transparent; }
                input[type='range']::-webkit-slider-runnable-track { width: 100%; height: 6px; cursor: pointer; background: hsl(var(--surface-raised)); border-radius: 3px; border: 1px solid var(--glass-border); }
                input[type='range']::-webkit-slider-thumb { -webkit-appearance: none; border: 2px solid hsl(var(--accent)); height: 18px; width: 18px; border-radius: 50%; background: hsl(var(--bg)); cursor: pointer; margin-top: -6px; box-shadow: 0 0 10px rgba(0,0,0,0.5); }
                
                .range-labels { display: flex; justify-content: space-between; margin-top: 0.25rem; font-size: 0.625rem; color: hsl(var(--text-dim)); font-weight: 800; text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.5; }
                
                .card-actions-row { margin-top: var(--s-16); }
                .btn-block { width: 100%; border-radius: var(--radius-md); font-weight: 850; letter-spacing: 0.1em; padding: var(--s-5); }

                /* Classification Animation */
                .classification-active { display: flex; align-items: center; justify-content: center; gap: var(--s-3); }
                .scanning-icon { color: hsl(var(--accent)); animation: spin 2s linear infinite; display: flex; }

                @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

                .meta-stats { display: flex; gap: var(--s-12); margin-top: var(--s-12); border-top: 1px solid var(--glass-border); padding-top: var(--s-8); width: 100%; justify-content: center; }
                .stat-box { display: flex; flex-direction: column; gap: 6px; text-align: center; }
                .stat-label { font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim)); text-transform: uppercase; letter-spacing: 0.12em; }
                .stat-value { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-muted)); font-family: var(--font-mono); }
                .stat-value.highlight { color: hsl(var(--accent)); }

                @media (max-width: 950px) {
                    .config-layout { grid-template-columns: 1fr; }
                    .preview-visual { min-height: 320px; }
                }
                "
            </style>
        </div>
    }
}

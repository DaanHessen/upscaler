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
    
    let (quality, set_quality) = signal("2K".to_string());
    let (style, set_style) = signal("PHOTOGRAPHY".to_string());
    let (temperature, set_temperature) = signal(0.0f32);
    let (loading, set_loading) = signal(false);

    // Update style based on classification when it's available
    Effect::new(move |_| {
        if let Some(cls) = global_state.temp_classification.get() {
            set_style.set(cls);
        }
    });

    let handle_upscale = move |_| {
        let navigate = navigate.clone();
        if let Some(file) = global_state.temp_file.get() {
            set_loading.set(true);
            let token = auth.session.get().map(|s| s.access_token);
            let q_val = quality.get();
            let s_val = style.get();
            let t_val = temperature.get();
            let set_credits = auth.set_credits;
            
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, token.as_deref()).await {
                    Ok(resp) => {
                        // Refresh balance
                        if let Ok(new_balance) = ApiClient::get_balance(token.as_deref()).await {
                            set_credits.set(Some(new_balance));
                        }
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
                <h1>"Refinement Params"</h1>
                <p class="muted">"Configure the neural reconstruction engine for your asset."</p>
            </div>

            <div class="config-layout">
                <div class="config-left">
                    <div class="card preview-card">
                        <div class="card-header">
                            <ImageIcon size={18} />
                            <span>"Source Asset"</span>
                        </div>
                        <div class="preview-visual">
                            {move || if global_state.temp_file.get().is_some() {
                                view! { <img src=preview_url /> }.into_any()
                            } else {
                                view! { <div class="empty-preview">"No image selected"</div> }.into_any()
                            }}
                        </div>
                        <div class="detection-info">
                            <Zap size={14} />
                            <span>"AUTOMATICALLY CLASSIFIED AS: " <strong>{move || global_state.temp_classification.get().unwrap_or("PENDING...".to_string())}</strong></span>
                        </div>
                    </div>
                </div>

                <div class="config-right">
                    <div class="card params-card">
                        <div class="card-header">
                            <Settings size={18} />
                            <span>"Parameters"</span>
                        </div>
                        
                        <div class="params-body">
                            <div class="param-group" title="Higher resolution requires more processing power and credits.">
                                <label>"Target Resolution"</label>
                                <div class="radio-group">
                                    <div 
                                        class=move || if quality.get() == "2K" { "radio-item active" } else { "radio-item" }
                                        on:click=move |_| set_quality.set("2K".to_string())
                                    >
                                        "2K (HD)"
                                        <span class="cost">"2 CREDITS"</span>
                                    </div>
                                    <div 
                                        class=move || if quality.get() == "4K" { "radio-item active" } else { "radio-item" }
                                        on:click=move |_| set_quality.set("4K".to_string())
                                    >
                                        "4K (UHD)"
                                        <span class="cost">"4 CREDITS"</span>
                                    </div>
                                </div>
                            </div>

                            <div class="param-group" title="Choose the model that best fits your image type.">
                                <label>"Model Reconstruction"</label>
                                <select on:change=move |ev| set_style.set(event_target_value(&ev)) prop:value=style>
                                    <option value="PHOTOGRAPHY">"Photography (Natural details)"</option>
                                    <option value="ILLUSTRATION">"Illustration (Sharp lines)"</option>
                                </select>
                            </div>

                            <div class="param-group" title="Temperature controls model variability. Lower is more faithful.">
                                <label>"Temperature: " {move || format!("{:.1}", temperature.get())}</label>
                                <input 
                                    type="range" 
                                    min="0.0" 
                                    max="2.0" 
                                    step="0.1" 
                                    prop:value=move || temperature.get().to_string()
                                    on:input=move |ev| set_temperature.set(event_target_value(&ev).parse().unwrap_or(0.0))
                                />
                                <div class="range-labels">
                                    <span>"Faithful"</span>
                                    <span>"Creative"</span>
                                </div>
                            </div>
                        </div>

                        <div class="card-footer">
                            <button 
                                class="btn btn-primary btn-lg btn-block" 
                                disabled=move || loading.get() || global_state.temp_file.get().is_none()
                                on:click=handle_upscale
                            >
                                {move || if loading.get() { "Enqueuing..." } else { "Begin Reconstruction" }}
                            </button>
                            <p class="cost-summary muted">
                                "Estimated Pipeline Time: ~15 seconds"
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".configure-page { max-width: 1100px; margin: 0 auto; }
                .config-layout { display: grid; grid-template-columns: 1fr 400px; gap: 2rem; margin-top: 2rem; align-items: flex-start; }
                
                .preview-visual { background: #000; display: flex; align-items: center; justify-content: center; min-height: 400px; border-radius: 4px; overflow: hidden; margin: 1rem; border: 1px solid var(--border-color); }
                .preview-visual img { max-width: 100%; max-height: 500px; }
                .empty-preview { color: var(--text-muted); font-size: 0.9rem; }
                
                .detection-info { padding: 1rem 1.5rem 1.5rem; display: flex; align-items: center; gap: 0.75rem; font-size: 0.7rem; color: var(--accent); font-weight: 700; letter-spacing: 0.05em; }
                .detection-info strong { color: var(--text-color); }

                .params-body { padding: 2rem; display: flex; flex-direction: column; gap: 2rem; }
                .param-group label { display: block; font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); margin-bottom: 0.75rem; }
                
                .radio-group { display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; }
                .radio-item { padding: 1rem; border: 1px solid var(--border-color); border-radius: 8px; cursor: pointer; transition: all 0.2s; font-size: 0.8rem; font-weight: 600; display: flex; flex-direction: column; gap: 0.25rem; }
                .radio-item:hover { border-color: var(--accent); background: var(--surface-lighter); }
                .radio-item.active { border-color: var(--accent); background: rgba(88, 166, 255, 0.1); color: var(--accent); }
                .radio-item .cost { font-size: 0.6rem; opacity: 0.6; font-weight: 800; font-family: var(--font-mono); }

                .range-labels { display: flex; justify-content: space-between; margin-top: 0.5rem; font-size: 0.65rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }
                
                .btn-block { width: 100%; }
                .card-footer { padding: 1.5rem 2rem 2rem; border-top: 1px solid var(--border-color); text-align: center; }
                .cost-summary { font-size: 0.7rem; margin-top: 1rem; }
                
                @media (max-width: 900px) {
                    .config-layout { grid-template-columns: 1fr; gap: 1rem; }
                    .preview-visual { min-height: 300px; margin: 0; border-radius: 0; border-left: none; border-right: none; }
                    .params-body { padding: 1.5rem; }
                    .card-footer { padding: 1.5rem; }
                    .radio-item { padding: 0.75rem; }
                }
                "
            </style>
        </div>
    }
}

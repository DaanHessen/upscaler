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
        <div class="settings-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1 class="stagger-1 text-gradient">"Upscale Settings"</h1>
                    <p class="muted stagger-2">"Configure restoration parameters for your asset."</p>
                </div>
            </div>

            // ── Main card: two-column grid, same pattern as Credits page ──
            <div class="card shadow-lg" style="margin-top: var(--s-8);">
                <div style="display: grid; grid-template-columns: 1fr 1fr;">

                    // ── Left: Preview ────────────────────────────────────
                    <div style="padding: var(--s-10) var(--s-12); border-right: 1px solid hsl(var(--border) / 0.5); display: flex; flex-direction: column;">
                        <div class="card-tag">
                            <ImageIcon size={10} />
                            <span>"ASSET PREVIEW"</span>
                        </div>

                        <div class="cfg-preview-box" style="margin-top: var(--s-8); flex: 1; display: flex; align-items: center; justify-content: center; background: hsl(var(--surface-raised)); border-radius: var(--radius-md); overflow: hidden; min-height: 300px;">
                            {move || {
                                if global_state.temp_file.get().is_some() {
                                    view! { <img src=preview_src() style="width: 100%; height: 100%; object-fit: contain; display: block;" /> }.into_any()
                                } else {
                                    view! {
                                        <div style="display: flex; flex-direction: column; align-items: center; gap: var(--s-3); opacity: 0.25; color: hsl(var(--text-dim));">
                                            <ImageIcon size={40} />
                                            <span style="font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em;">"No asset loaded"</span>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        <div class="meta-stats" style="margin-top: var(--s-8); padding-top: var(--s-6); border-top: 1px solid hsl(var(--border-muted));">
                            <div class="stat-box">
                                <span class="stat-label">"Detected Style"</span>
                                <span class="stat-value" style="font-size: 0.9375rem;">
                                    {move || global_state.temp_classification.get().unwrap_or_else(|| "—".to_string())}
                                </span>
                            </div>
                            <div class="stat-box">
                                <span class="stat-label">"Status"</span>
                                <span class="stat-value highlight" style="font-size: 0.9375rem;">"READY"</span>
                            </div>
                        </div>
                    </div>

                    // ── Right: Settings ──────────────────────────────────
                    <div style="padding: var(--s-10) var(--s-12); display: flex; flex-direction: column;">
                        <div class="card-tag" style="margin-bottom: var(--s-8);">
                            <Settings size={10} />
                            <span>"PARAMETERS"</span>
                        </div>

                        // Resolution
                        <div style="display: grid; gap: var(--s-4);">
                            <div class="data-row">
                                <span class="data-label">"TARGET RESOLUTION"</span>
                            </div>
                            <div class="pack-list">
                                <div
                                    class=move || if global_state.quality.get() == "2K" { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_quality.set("2K".to_string())
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"2K — HD Restore"</span>
                                        <span class="pack-credits">"STANDARD QUALITY"</span>
                                    </div>
                                    <span class="pack-price">"2C"</span>
                                </div>
                                <div
                                    class=move || if global_state.quality.get() == "4K" { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"4K — Ultra HD"</span>
                                        <span class="pack-credits">"MAXIMUM QUALITY"</span>
                                    </div>
                                    <span class="pack-price">"4C"</span>
                                </div>
                            </div>
                        </div>

                        // Style
                        <div style="display: grid; gap: var(--s-4); margin-top: var(--s-8);">
                            <div class="data-row">
                                <span class="data-label">"RECONSTRUCTION STYLE"</span>
                            </div>
                            <div class="pack-list">
                                <div
                                    class=move || if global_state.style.get() == "PHOTOGRAPHY" { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"Photography"</span>
                                        <span class="pack-credits">"OPTIMIZED FOR PHOTOS"</span>
                                    </div>
                                </div>
                                <div
                                    class=move || if global_state.style.get() == "ILLUSTRATION" { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"Illustration"</span>
                                        <span class="pack-credits">"OPTIMIZED FOR ART"</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Temperature
                        <div style="margin-top: var(--s-8); padding-top: var(--s-8); border-top: 1px solid hsl(var(--border-muted));">
                            <div class="data-row" style="margin-bottom: var(--s-4);">
                                <span class="data-label">"CREATIVE DRIFT"</span>
                                <span class="data-value" style="color: hsl(var(--accent));">{move || format!("{:.1}", global_state.temperature.get())}</span>
                            </div>
                            <input
                                type="range"
                                min="0.0"
                                max="2.0"
                                step="0.1"
                                style="width: 100%;"
                                prop:value=move || global_state.temperature.get().to_string()
                                on:input=move |ev| global_state.set_temperature.set(leptos::prelude::event_target_value(&ev).parse().unwrap_or(0.0))
                            />
                        </div>

                        // Lighting
                        <div style="margin-top: var(--s-8);">
                            <div class="data-row" style="margin-bottom: var(--s-4);">
                                <span class="data-label">"ATMOSPHERIC LIGHTING"</span>
                            </div>
                            <select
                                style="width: 100%; padding: var(--s-3) var(--s-4); background: hsl(var(--surface-raised)); border: 1px solid hsl(var(--border)); border-radius: var(--radius-sm); color: hsl(var(--text)); font-size: 0.875rem; font-weight: 600;"
                                on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                prop:value=move || global_state.lighting.get()
                            >
                                <option value="Original">"Maintain Original"</option>
                                <option value="Studio">"Studio Lighting"</option>
                                <option value="Cinematic">"Cinematic Shadowing"</option>
                                <option value="Vivid">"High Vividity"</option>
                                <option value="Natural">"Soft Ambient"</option>
                            </select>
                        </div>

                        // Preserves
                        <div style="margin-top: var(--s-8); padding-top: var(--s-8); border-top: 1px solid hsl(var(--border-muted));">
                            <div class="data-row" style="margin-bottom: var(--s-4);">
                                <span class="data-label">"ADVANCED PRESERVES"</span>
                            </div>
                            <div class="pack-list">
                                <div
                                    class=move || if global_state.keep_aspect_ratio.get() { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_keep_aspect_ratio.update(|v| *v = !*v)
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"Ratio Lock"</span>
                                        <span class="pack-credits">"MAINTAIN ORIGINAL ASPECT RATIO"</span>
                                    </div>
                                    <span class="pack-price" style="font-size: 0.75rem;">
                                        {move || if global_state.keep_aspect_ratio.get() { "ON" } else { "OFF" }}
                                    </span>
                                </div>
                                <div
                                    class=move || if global_state.keep_depth_of_field.get() { "pack-item active" } else { "pack-item" }
                                    on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                >
                                    <div class="pack-info">
                                        <span class="pack-name">"Depth Lock"</span>
                                        <span class="pack-credits">"PRESERVE DEPTH OF FIELD"</span>
                                    </div>
                                    <span class="pack-price" style="font-size: 0.75rem;">
                                        {move || if global_state.keep_depth_of_field.get() { "ON" } else { "OFF" }}
                                    </span>
                                </div>
                            </div>
                        </div>

                        // Action
                        <div style="display: flex; justify-content: center; margin-top: auto; padding-top: var(--s-10);">
                            <button
                                class="btn btn-primary btn-lg"
                                style="width: 100%; font-size: 0.875rem; font-weight: 800; padding: var(--s-5) 0; gap: var(--s-3);"
                                disabled=move || loading.get() || global_state.temp_file.get().is_none()
                                on:click=handle_upscale
                            >
                                <Zap size={16} />
                                {move || if loading.get() { "STARTING ENGINE..." } else { "INITIATE UPSCALE" }}
                                <span class="user-badge" style="margin-left: var(--s-2);">
                                    {move || if global_state.quality.get() == "4K" { "4 CREDITS" } else { "2 CREDITS" }}
                                </span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                "@media (max-width: 900px) {
                    .settings-container .card > div[style*='grid-template-columns'] {
                        grid-template-columns: 1fr !important;
                    }
                    .settings-container .card > div > div[style*='border-right'] {
                        border-right: none !important;
                        border-bottom: 1px solid hsl(var(--border) / 0.5);
                    }
                }
                "
            </style>
        </div>
    }
}

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings};
use crate::components::icons::{Zap, ImageIcon, Settings, Target, Sun};

#[component]
pub fn Configure() -> impl IntoView {
    let global_state = use_global_state();
    let auth = use_auth();
    let navigate = use_navigate();
    
    let (loading, set_loading) = signal(false);

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
                keep_depth_of_field: global_state.keep_depth_of_field.get(),
                lighting: global_state.lighting.get(),
                thinking_level: global_state.thinking_level.get(),
            };
            
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, &p_settings, token.as_deref()).await {
                    Ok(resp) => {
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

            <div class="card shadow-lg stagger-3" style="margin-top: var(--s-8); overflow: hidden;">
                <div class="cfg-grid">

                    // ── Left: Preview ──
                    <div class="cfg-left">
                        <div class="card-tag">
                            <ImageIcon size={10} />
                            <span>"ASSET PREVIEW"</span>
                        </div>

                        <div class="cfg-img-box">
                            {move || {
                                if global_state.temp_file.get().is_some() {
                                    view! { <img src=preview_src() class="cfg-img" /> }.into_any()
                                } else {
                                    view! {
                                        <div class="cfg-empty">
                                            <ImageIcon size={28} />
                                            <span>"No asset"</span>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        <div class="cfg-detected">
                            <span class="cfg-det-label">"DETECTED"</span>
                            <span class="cfg-det-value">
                                {move || global_state.temp_classification.get().unwrap_or_else(|| "—".to_string())}
                            </span>
                        </div>
                    </div>

                    // ── Right: Settings ──
                    <div class="cfg-right">
                        <div class="card-tag" style="margin-bottom: var(--s-5);">
                            <Settings size={10} />
                            <span>"PARAMETERS"</span>
                        </div>

                        // Resolution
                        <div class="cfg-section">
                            <span class="cfg-label">"RESOLUTION"</span>
                            <div class="cfg-row-2">
                                <div
                                    class=move || if global_state.quality.get() == "2K" { "cfg-opt active" } else { "cfg-opt" }
                                    on:click=move |_| global_state.set_quality.set("2K".to_string())
                                >
                                    <strong>"2K"</strong>
                                    <span>"2 Credits"</span>
                                </div>
                                <div
                                    class=move || if global_state.quality.get() == "4K" { "cfg-opt active" } else { "cfg-opt" }
                                    on:click=move |_| global_state.set_quality.set("4K".to_string())
                                >
                                    <strong>"4K"</strong>
                                    <span>"4 Credits"</span>
                                </div>
                            </div>
                        </div>

                        // Style
                        <div class="cfg-section">
                            <span class="cfg-label">"STYLE"</span>
                            <div class="cfg-row-2">
                                <div
                                    class=move || if global_state.style.get() == "PHOTOGRAPHY" { "cfg-opt active" } else { "cfg-opt" }
                                    on:click=move |_| global_state.set_style.set("PHOTOGRAPHY".to_string())
                                >
                                    <strong>"Photo"</strong>
                                </div>
                                <div
                                    class=move || if global_state.style.get() == "ILLUSTRATION" { "cfg-opt active" } else { "cfg-opt" }
                                    on:click=move |_| global_state.set_style.set("ILLUSTRATION".to_string())
                                >
                                    <strong>"Illustration"</strong>
                                </div>
                            </div>
                        </div>

                        // Creative Drift + Depth Lock (side by side)
                        <div class="cfg-section-row">
                            <div class="cfg-section" style="flex: 1;">
                                <div style="display: flex; justify-content: space-between; align-items: center;">
                                    <span class="cfg-label">"CREATIVE DRIFT"</span>
                                    <span class="cfg-val">{move || format!("{:.1}", global_state.temperature.get())}</span>
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
                            <div class="cfg-section" style="flex: 1;">
                                <span class="cfg-label">"DEPTH LOCK"</span>
                                <div
                                    class=move || if global_state.keep_depth_of_field.get() { "cfg-opt compact active" } else { "cfg-opt compact" }
                                    on:click=move |_| global_state.set_keep_depth_of_field.update(|v| *v = !*v)
                                >
                                    <Target size={14} />
                                    <strong>{move || if global_state.keep_depth_of_field.get() { "Enabled" } else { "Disabled" }}</strong>
                                </div>
                            </div>
                        </div>

                        // Lighting
                        <div class="cfg-section">
                            <span class="cfg-label">"LIGHTING"</span>
                            <select
                                class="cfg-select"
                                on:change=move |ev| global_state.set_lighting.set(leptos::prelude::event_target_value(&ev))
                                prop:value=move || global_state.lighting.get()
                            >
                                <option value="Original">"Original"</option>
                                <option value="Studio">"Studio"</option>
                                <option value="Cinematic">"Cinematic"</option>
                                <option value="Vivid">"Vivid"</option>
                                <option value="Natural">"Natural"</option>
                            </select>
                        </div>

                        // Action
                        <button
                            class="btn btn-primary btn-lg cfg-submit"
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

            <style>
                "/* ── Configure grid ───────────────────── */
                .cfg-grid {
                    display: grid;
                    grid-template-columns: 280px 1fr;
                }

                /* ── Left column: preview ─────────────── */
                .cfg-left {
                    padding: var(--s-8);
                    border-right: 1px solid hsl(var(--border) / 0.5);
                    display: flex;
                    flex-direction: column;
                    gap: var(--s-4);
                }

                .cfg-img-box {
                    flex: 1;
                    min-height: 200px;
                    background: hsl(var(--surface-raised));
                    border-radius: var(--radius-md);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    overflow: hidden;
                }

                .cfg-img {
                    width: 100%;
                    height: 100%;
                    object-fit: contain;
                }

                .cfg-empty {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: var(--s-2);
                    color: hsl(var(--text-dim));
                    opacity: 0.25;
                    font-size: 0.6875rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 0.1em;
                }

                .cfg-detected {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding-top: var(--s-4);
                    border-top: 1px solid hsl(var(--border-muted));
                }

                .cfg-det-label {
                    font-size: 0.5625rem;
                    font-weight: 900;
                    color: hsl(var(--text-dim));
                    letter-spacing: 0.12em;
                    text-transform: uppercase;
                }

                .cfg-det-value {
                    font-size: 0.75rem;
                    font-weight: 700;
                    color: hsl(var(--accent));
                    font-family: var(--font-mono);
                    text-transform: uppercase;
                }

                /* ── Right column: controls ───────────── */
                .cfg-right {
                    padding: var(--s-8);
                    display: flex;
                    flex-direction: column;
                    gap: var(--s-5);
                }

                .cfg-section {
                    display: flex;
                    flex-direction: column;
                    gap: var(--s-3);
                }

                .cfg-section-row {
                    display: flex;
                    gap: var(--s-5);
                }

                .cfg-label {
                    font-size: 0.5625rem;
                    font-weight: 900;
                    color: hsl(var(--text-dim));
                    letter-spacing: 0.14em;
                    text-transform: uppercase;
                }

                .cfg-val {
                    font-family: var(--font-mono);
                    font-size: 0.75rem;
                    font-weight: 800;
                    color: hsl(var(--accent));
                }

                /* ── Selection tiles ──────────────────── */
                .cfg-row-2 {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: var(--s-3);
                }

                .cfg-opt {
                    background: hsl(var(--surface-raised));
                    border: 1px solid hsl(var(--border));
                    border-radius: var(--radius-sm);
                    padding: var(--s-3) var(--s-4);
                    cursor: pointer;
                    transition: all 0.15s ease;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: 2px;
                    text-align: center;
                }

                .cfg-opt strong {
                    font-size: 0.8125rem;
                    font-weight: 800;
                    color: hsl(var(--text));
                    letter-spacing: -0.01em;
                }

                .cfg-opt span {
                    font-size: 0.5625rem;
                    font-weight: 700;
                    color: hsl(var(--text-dim));
                    text-transform: uppercase;
                    letter-spacing: 0.08em;
                }

                .cfg-opt:hover:not(.active) {
                    border-color: hsl(var(--accent) / 0.4);
                }

                .cfg-opt.active {
                    background: hsl(var(--accent) / 0.1);
                    border-color: hsl(var(--accent));
                }

                .cfg-opt.active strong {
                    color: hsl(var(--accent));
                }

                .cfg-opt.active span {
                    color: hsl(var(--accent) / 0.7);
                }

                .cfg-opt.compact {
                    flex-direction: row;
                    justify-content: center;
                    gap: var(--s-3);
                    padding: var(--s-4);
                }

                .cfg-opt.compact strong {
                    font-size: 0.75rem;
                }

                /* ── Select ───────────────────────────── */
                .cfg-select {
                    width: 100%;
                    padding: var(--s-3) var(--s-4);
                    background: hsl(var(--surface-raised));
                    border: 1px solid hsl(var(--border));
                    border-radius: var(--radius-sm);
                    color: hsl(var(--text));
                    font-size: 0.8125rem;
                    font-weight: 600;
                    appearance: none;
                    cursor: pointer;
                    transition: border-color 0.15s;
                }

                .cfg-select:hover {
                    border-color: hsl(var(--accent) / 0.5);
                }

                /* ── Submit ───────────────────────────── */
                .cfg-submit {
                    margin-top: auto;
                    width: 100%;
                    gap: var(--s-3);
                }

                /* ── Responsive ───────────────────────── */
                @media (max-width: 800px) {
                    .cfg-grid {
                        grid-template-columns: 1fr;
                    }
                    .cfg-left {
                        border-right: none;
                        border-bottom: 1px solid hsl(var(--border) / 0.5);
                    }
                    .cfg-img-box {
                        min-height: 160px;
                        max-height: 240px;
                    }
                    .cfg-section-row {
                        flex-direction: column;
                    }
                }
                "
            </style>
        </div>
    }
}

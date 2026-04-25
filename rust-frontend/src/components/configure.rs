use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings};
use crate::components::icons::{Zap, ImageIcon, Target, RefreshCw, Download, ZoomIn, ZoomOut, Info};

#[component]
pub fn Configure() -> impl IntoView {
    let gs = use_global_state();
    let auth = use_auth();
    let _navigate = use_navigate();

    let (is_dragging, set_is_dragging) = signal(false);

    let (before_url, set_before_url) = signal(Option::<String>::None);
    let (view_mode, set_view_mode) = signal("compare".to_string());
    let (zoom_level, set_zoom_level) = signal(1.0f64);

    Effect::new(move |_| {
        if let Some(file) = gs.temp_file.get() {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                set_before_url.set(Some(url.clone()));
                on_cleanup(move || {
                    let _ = web_sys::Url::revoke_object_url(&url);
                });
            }
        } else {
            set_before_url.set(None);
        }
    });

    let on_file_input = move |ev: leptos::web_sys::Event| {
        let input: web_sys::HtmlInputElement = leptos::prelude::event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                if file.size() > 20_000_000.0 {
                    gs.show_error("File exceeds 20MB limit.");
                    return;
                }
                if !file.type_().starts_with("image/") {
                    gs.show_error("Invalid file type. Only images are allowed.");
                    return;
                }
                gs.clear_notification();
                gs.set_temp_file.set(Some(file));
                gs.set_preview_base64.set(None);
            }
        }
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
        if let Some(data) = ev.data_transfer() {
            if let Some(files) = data.files() {
                if let Some(file) = files.get(0) {
                    if file.size() > 20_000_000.0 {
                        gs.show_error("File exceeds 20MB limit.");
                        return;
                    }
                    if !file.type_().starts_with("image/") {
                        gs.show_error("Invalid file type. Only images are allowed.");
                        return;
                    }
                    gs.clear_notification();
                    gs.set_temp_file.set(Some(file));
                    gs.set_preview_base64.set(None);
                }
            }
        }
    };

    let handle_upscale = move |_| {
        if let Some(file) = gs.temp_file.get() {
            let q_val: String = gs.quality.get();
            let cost = if q_val == "4K" { 4 } else { 2 };
            if let Some(current) = auth.credits.get() {
                if current < cost {
                    gs.show_error("Insufficient credits.");
                    return;
                }
            }
            
            gs.set_is_submitting.set(true);
            gs.clear_notification();
            let token = auth.session.get().map(|s| s.access_token);
            let s_val: String = gs.style.get();
            let t_val: f32 = gs.temperature.get();
            let tool_val: String = gs.active_tool.get();
            let auth_ctx = auth;
            let state = gs;
            let p_settings = PromptSettings {
                keep_depth_of_field: gs.keep_depth_of_field.get(),
                lighting: gs.lighting.get(),
                thinking_level: gs.thinking_level.get(),
                seed: gs.seed.get(),
                target_medium: gs.target_medium.get(),
                render_style: gs.render_style.get(),
                target_aspect_ratio: gs.target_aspect_ratio.get(),
                refinement_pass: gs.refinement_pass.get(),
                debug_gemini_only: gs.debug_gemini_only.get(),
            };
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &q_val, &s_val, t_val, &p_settings, &tool_val, token.as_deref()).await {
                    Ok(resp) => {
                        auth_ctx.set_credits.update(|c| if let Some(cv) = c { *cv -= cost; });
                        auth_ctx.sync_telemetry(true);
                        state.set_processing_job.set(Some(resp.job_id));
                        state.set_is_submitting.set(false);
                    },
                    Err(e) => { 
                        state.show_error(format!("Upload failed: {}", e)); 
                        state.set_is_submitting.set(false);
                    }
                }
            });
        }
    };

    let handle_try_again = move |_| {
        gs.set_preview_base64.set(None);
        gs.set_engine_status.set(None);
        gs.set_processing_job.set(None);
        set_zoom_level.set(1.0);
    };

    let stage_info = move || {
        if gs.is_submitting.get() {
            return ("INIT", "Uploading image", "Preprocessing image...");
        }
        match gs.engine_status.get().map(|s| s.status) {
            Some(s) if s == "PENDING"    => ("QUEUE",  "System Ready",   "Checking on safety filters..."),
            Some(s) if s == "PROCESSING" => ("ACTIVE", "Upscaling Image", "Rescaling image..."),
            Some(s) if s == "COMPLETED"  => ("DONE",   "Export Ready",   "Enhancement complete. Ready for download."),
            _ =>                            ("IDLE",   "Standby",        "Awaiting engine handshake."),
        }
    };

    view! {
        <div class="editor-shell fade-in">

            // ── Canvas ──────────────────────────────────────────
            <div class="editor-canvas-area"
                 on:dragover=move |ev| { ev.prevent_default(); set_is_dragging.set(true); }
                 on:dragleave=move |_| set_is_dragging.set(false)
                 on:drop=on_drop
            >
                <div class="canvas-grid"></div>

                {move || {
                    let preview = gs.preview_base64.get();
                    let before = before_url.get();
                    
                    match (before, preview) {
                        (Some(before), Some(after)) => view! {
                            <div class="canvas-view-container animate-in">
                                <div class="slider-fill">
                                    <crate::components::comparison_slider::ComparisonSlider 
                                        images=vec![(before, after.clone())] 
                                        zoom=zoom_level.get()
                                        view_mode=view_mode
                                    />
                                </div>

                                // Viewer Controls
                                <div class="viewer-controls">
                                    <div class="viewer-ctrl-group">
                                        <button 
                                            class:active=move || view_mode.get() == "compare"
                                            on:click=move |_| set_view_mode.set("compare".to_string())
                                        >"Compare"</button>
                                        <button 
                                            class:active=move || view_mode.get() == "original"
                                            on:click=move |_| set_view_mode.set("original".to_string())
                                        >"Original"</button>
                                        <button 
                                            class:active=move || view_mode.get() == "upscaled"
                                            on:click=move |_| set_view_mode.set("upscaled".to_string())
                                        >"Upscaled"</button>
                                    </div>
                                    
                                    <div class="viewer-ctrl-divider"></div>

                                    <div class="viewer-ctrl-group">
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z + 0.5).min(4.0))
                                            title="Zoom In"
                                        ><ZoomIn size={14} /></button>
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z - 0.5).max(1.0))
                                            title="Zoom Out"
                                        ><ZoomOut size={14} /></button>
                                    </div>

                                    <div class="viewer-ctrl-divider"></div>

                                    <div class="viewer-ctrl-group">
                                        <a 
                                            href=after.clone() 
                                            target="_blank" 
                                            class="viewer-action-btn"
                                            style="text-decoration:none;"
                                        >
                                            <Download size={14} />
                                            <span>"Download"</span>
                                        </a>
                                        <button 
                                            class="viewer-action-btn secondary"
                                            on:click=handle_try_again
                                        >
                                            <RefreshCw size={14} />
                                            <span>"Again"</span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        (Some(before), None) => view! {
                            <div class="canvas-view-container preview-only">
                                <div class="asset-wrapper">
                                    <div class="corner-accents"><div class="corner tl"></div><div class="corner tr"></div><div class="corner bl"></div><div class="corner br"></div></div>
                                    <img class="studio-asset" src=before alt="Original asset" style=move || format!("transform: scale({})", zoom_level.get()) />
                                </div>
                                
                                <div class="viewer-controls">
                                    <div class="viewer-ctrl-group">
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z + 0.5).min(4.0))
                                            title="Zoom In"
                                        ><ZoomIn size={14} /></button>
                                        <button 
                                            on:click=move |_| set_zoom_level.update(|z| *z = (*z - 0.5).max(1.0))
                                            title="Zoom Out"
                                        ><ZoomOut size={14} /></button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        _ => {
                            let file_input_ref = NodeRef::<leptos::html::Input>::new();
                            view! {
                                <div class="drop-zone-outer stagger-1" on:click=move |_| {
                                    if let Some(input) = file_input_ref.get() {
                                        let _ = input.click();
                                    }
                                }>
                                    <div class="drop-zone-inner">
                                        <div class="dz-icon-wrap">
                                            <ImageIcon size={32} />
                                        </div>
                                        <h2 class="dz-title">{crate::text::TXT.editor_empty_title}</h2>
                                        <p class="dz-sub">{crate::text::TXT.editor_empty_desc}</p>
                                        <div class="dz-formats">"JPG, PNG, WEBP (MAX 20MB)"</div>
                                        
                                        <input type="file" accept="image/*" style="display: none;" 
                                               on:change=on_file_input node_ref=file_input_ref />
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }}

                <Show when=move || is_dragging.get()>
                    <div class="drag-overlay">
                        <div class="drag-box">
                            <Download size={32} />
                            <span>"Drop to import asset"</span>
                        </div>
                    </div>
                </Show>
            </div>

            // ── Sidebar ─────────────────────────────────────────
            <aside class="editor-sidebar">
                {move || {
                    let submitting = gs.is_submitting.get();
                    let job = gs.processing_job.get();
                    
                    if submitting || job.is_some() {
                        view! {
                            <div class="sidebar-inner fade-in">
                                <div class="polling-view">
                                    <div class="polling-header">
                                        <div class="p-status-pill">
                                            <div class="status-dot pulse"></div>
                                            <span>{move || stage_info().0}</span>
                                        </div>
                                        <h3 class="p-title">{move || stage_info().1}</h3>
                                        <p class="p-desc">{move || stage_info().2}</p>
                                    </div>

                                    <div class="p-progress-rail">
                                        <div class="p-progress-fill" style:width=move || {
                                            if submitting { return "15%".to_string(); }
                                            match gs.engine_status.get().map(|s| s.status) {
                                                Some(s) if s == "PENDING" => "40%".to_string(),
                                                Some(s) if s == "PROCESSING" => "75%".to_string(),
                                                Some(s) if s == "COMPLETED" => "100%".to_string(),
                                                _ => "10%".to_string()
                                            }
                                        }></div>
                                    </div>
                                    
                                    <div class="p-telemetry-grid">
                                        <div class="p-tele-item">
                                            <span class="p-tele-label">"LATENCY"</span>
                                            <span class="p-tele-val">{move || gs.engine_status.get().and_then(|s| s.latency_ms).map(|l| format!("{}ms", l)).unwrap_or_else(|| "---".to_string())}</span>
                                        </div>
                                        <div class="p-tele-item">
                                            <span class="p-tele-label">"TOKEN"</span>
                                            <span class="p-tele-val">{move || job.map(|id| id.to_string().chars().take(8).collect::<String>()).unwrap_or_else(|| "---".to_string())}</span>
                                        </div>
                                    </div>

                                    <div class="p-footer">
                                        <p class="p-hint">"Please keep this tab open while your asset is being processed."</p>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="sidebar-inner fade-in">
                                <div class="sb-scroll">

                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Target size={10} />
                                                <span>"RESOLUTION"</span>
                                            </div>
                                                <div class="res-grid">
                                                    <div
                                                        class=move || if gs.quality.get() == "2K" { "pack-item active" } else { "pack-item" }
                                                        on:click=move |_| gs.set_quality.set("2K".to_string())
                                                    >
                                                        <div class="pack-info">
                                                            <span class="res-big-num">"2K"</span>
                                                            <span class="pack-price">"2 credits"</span>
                                                        </div>
                                                    </div>
                                                    <div
                                                        class=move || if gs.quality.get() == "4K" { "pack-item active" } else { "pack-item" }
                                                        on:click=move |_| gs.set_quality.set("4K".to_string())
                                                    >
                                                        <div class="pack-info">
                                                            <span class="res-big-num">"4K"</span>
                                                            <span class="pack-price">"4 credits"</span>
                                                        </div>
                                                    </div>
                                                </div>
                                        </div>
                                    </div>

                                    // ── Engine ──────────────────────
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                                <Zap size={10} />
                                                <span>"ENGINE"</span>
                                            </div>

                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <label class="sb-label" style="display: flex; align-items: center;">"STYLE"<span title="The visual style of the reconstructed image. Use Illustration for drawings/art." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button 
                                                        class:active=move || gs.style.get() == "PHOTOGRAPHY"
                                                        on:click=move |_| gs.set_style.set("PHOTOGRAPHY".to_string())
                                                    >
                                                        "Photography"
                                                    </button>
                                                    <button 
                                                        class:active=move || gs.style.get() == "ILLUSTRATION"
                                                        on:click=move |_| gs.set_style.set("ILLUSTRATION".to_string())
                                                    >
                                                        "Illustration"
                                                    </button>
                                                </div>
                                            </div>

                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <div class="sb-label-row">
                                                    <label class="sb-label" style="display: flex; align-items: center;">"CREATIVITY"<span title="0.0 for perfectly faithful restoration, higher values add micro-details or hallucinations." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                    <span class="sb-val-badge">{move || format!("{:.1}", gs.temperature.get())}</span>
                                                </div>
                                                <input 
                                                    type="range" min="0.0" max="2.0" step="0.1" 
                                                    class="studio-slider"
                                                    prop:value=move || gs.temperature.get()
                                                    on:input=move |ev| gs.set_temperature.set(event_target_value(&ev).parse().unwrap_or(0.0))
                                                />
                                                <div class="slider-ends">
                                                    <span>"STRICT"</span>
                                                    <span>"CREATIVE"</span>
                                                </div>
                                            </div>

                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <label class="sb-label" style="display: flex; align-items: center;">"PROCESSING DEPTH"<span title="Higher depth allows the AI to perform a deeper analysis to remove compression artifacts before upscaling." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button 
                                                        class:active=move || gs.thinking_level.get() == "MINIMAL"
                                                        on:click=move |_| gs.set_thinking_level.set("MINIMAL".to_string())
                                                    >"Standard"</button>
                                                    <button 
                                                        class:active=move || gs.thinking_level.get() == "HIGH"
                                                        on:click=move |_| gs.set_thinking_level.set("HIGH".to_string())
                                                    >"Deep"</button>
                                                </div>
                                            </div>

                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <label class="sb-label" style="display: flex; align-items: center;">"REFINEMENT PASS"<span title="Preprocess the image to reduce artifacts and prepare it for upscaling." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button 
                                                        class:active=move || gs.refinement_pass.get() == false
                                                        on:click=move |_| gs.set_refinement_pass.set(false)
                                                    >"Off"</button>
                                                    <button 
                                                        class:active=move || gs.refinement_pass.get() == true
                                                        on:click=move |_| gs.set_refinement_pass.set(true)
                                                    >"On"</button>
                                                </div>
                                            </div>

                                            <Show when=move || gs.refinement_pass.get()>
                                                <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                    <label class="sb-label" style="display: flex; align-items: center;">"DEBUG GEMINI ONLY"<span title="Skip Topaz to preview the refinement pass." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                    <div class="seg-control">
                                                        <button 
                                                            class:active=move || gs.debug_gemini_only.get() == false
                                                            on:click=move |_| gs.set_debug_gemini_only.set(false)
                                                        >"Off"</button>
                                                        <button 
                                                            class:active=move || gs.debug_gemini_only.get() == true
                                                            on:click=move |_| gs.set_debug_gemini_only.set(true)
                                                        >"On"</button>
                                                    </div>
                                                </div>
                                            </Show>

                                            <div class="sb-field">
                                                <div class="sb-label-row">
                                                    <label class="sb-label" style="display: flex; align-items: center;">"RECONSTRUCTION SEED"<span title="Use a specific seed number to reproduce exactly the same results across identical upscales." style="cursor: help; margin-left: 4px; display: inline-flex; align-items: center;"><Info size={12} /></span></label>
                                                    <span class="sb-val-badge">{move || if let Some(s) = gs.seed.get() { s.to_string() } else { "AUTO".to_string() }}</span>
                                                </div>
                                                <div class="seed-row">
                                                    <input 
                                                        class="sb-input"
                                                        type="number" placeholder="Leave empty for auto" 
                                                        on:input=move |ev| {
                                                            let v = event_target_value(&ev);
                                                            gs.set_seed.set(if v.is_empty() { None } else { v.parse().ok() });
                                                        }
                                                        prop:value=move || gs.seed.get().map(|s| s.to_string()).unwrap_or_default()
                                                    />
                                                    <button class="seed-rng-btn" on:click=move |_| gs.set_seed.set(None) title="Reset to Auto"><RefreshCw size={14} /></button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="sb-footer">
                                    <button 
                                        class="btn btn-primary sb-cta" 
                                        disabled=move || gs.temp_file.get().is_none() || gs.is_submitting.get()
                                        on:click=handle_upscale
                                    >
                                        <div class="sb-cta-inner">
                                            <Zap size={16} />
                                            <span>"UPSCALE ASSET"</span>
                                        </div>
                                        <div class="sb-cta-badge">
                                            {move || if gs.quality.get() == "4K" { "4" } else { "2" }}
                                            <span style="font-size:0.5rem; opacity:0.6; margin-left:2px;">"CR"</span>
                                        </div>
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </aside>
        </div>
    }
}

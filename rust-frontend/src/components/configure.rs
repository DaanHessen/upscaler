use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::{use_global_state, use_auth};
use crate::api::{ApiClient, PromptSettings};
use crate::components::icons::{Zap, ImageIcon, Target, RefreshCw, Download, ZoomIn, ZoomOut, Info, Settings, ChevronDown};

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

    let cost = move || {
        let base_cost = match gs.scale.get().as_str() {
            "4x" => 4,
            "6x" => 6,
            _ => 2,
        };
        let pre_cost = if gs.pre_process_pass.get() { 1 } else { 0 };
        let rest_cost = if gs.restoration_pass.get() { 1 } else { 0 };
        let face_cost = if gs.face_enhancement.get() { 1 } else { 0 };
        base_cost + pre_cost + rest_cost + face_cost
    };

    let handle_upscale = move |_| {
        if let Some(file) = gs.temp_file.get() {
            let total_cost = cost();
            
            if let Some(current) = auth.credits.get() {
                if current < total_cost {
                    gs.show_error(format!("Insufficient credits. Need {}.", total_cost));
                    return;
                }
            }
            
            gs.set_is_submitting.set(true);
            gs.clear_notification();
            let token = auth.session.get().map(|s| s.access_token);
            let auth_ctx = auth;
            let state = gs;
            let p_settings = PromptSettings {
                pre_process_pass: gs.pre_process_pass.get(),
                restoration_pass: gs.restoration_pass.get(),
                face_enhancement: gs.face_enhancement.get(),
                creativity: gs.creativity.get(),
                seed: gs.seed.get(),
                noise_reduction: gs.noise_reduction.get(),
                sharpen: gs.sharpen.get(),
                remove_artifacts: gs.remove_artifacts.get(),
            };
            let scale_val = gs.scale.get();
            leptos::task::spawn_local(async move {
                match ApiClient::submit_upscale(&file, &scale_val, &p_settings, token.as_deref()).await {
                    Ok(resp) => {
                        auth_ctx.set_credits.update(|c| if let Some(cv) = c { *cv -= total_cost; });
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
            return ("INIT", "Uploading asset", "Preparing engine...");
        }
        match gs.engine_status.get().map(|s| s.status) {
            Some(s) if s == "PENDING"    => ("QUEUE",  "In Queue",   "Awaiting GPU allocation..."),
            Some(s) if s == "PROCESSING" => ("ACTIVE", "Processing", "Applying AI enhancement..."),
            Some(s) if s == "COMPLETED"  => ("DONE",   "Complete",   "Optimization finished."),
            _ =>                            ("IDLE",   "Standby",    "Ready to process."),
        }
    };

    view! {
        <div class="editor-shell fade-in">
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

                                <div class="viewer-controls">
                                    <div class="viewer-ctrl-group">
                                        <button class:active=move || view_mode.get() == "compare" on:click=move |_| set_view_mode.set("compare".to_string())>"Compare"</button>
                                        <button class:active=move || view_mode.get() == "original" on:click=move |_| set_view_mode.set("original".to_string())>"Original"</button>
                                        <button class:active=move || view_mode.get() == "upscaled" on:click=move |_| set_view_mode.set("upscaled".to_string())>"Upscaled"</button>
                                    </div>
                                    <div class="viewer-ctrl-divider"></div>
                                    <div class="viewer-ctrl-group">
                                        <button on:click=move |_| set_zoom_level.update(|z| *z = (*z + 0.5).min(4.0))><ZoomIn size={14} /></button>
                                        <button on:click=move |_| set_zoom_level.update(|z| *z = (*z - 0.5).max(1.0))><ZoomOut size={14} /></button>
                                    </div>
                                    <div class="viewer-ctrl-divider"></div>
                                    <div class="viewer-ctrl-group">
                                        <a href=after.clone() target="_blank" class="viewer-action-btn" style="text-decoration:none;"><Download size={14} /><span>"Download"</span></a>
                                        <button class="viewer-action-btn secondary" on:click=handle_try_again><RefreshCw size={14} /><span>"Again"</span></button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        (Some(before), None) => view! {
                            <div class="canvas-view-container preview-only">
                                <div class="asset-wrapper">
                                    <img class="studio-asset" src=before style=move || format!("transform: scale({})", zoom_level.get()) />
                                </div>
                                <div class="viewer-controls">
                                    <div class="viewer-ctrl-group">
                                        <button on:click=move |_| set_zoom_level.update(|z| *z = (*z + 0.5).min(4.0))><ZoomIn size={14} /></button>
                                        <button on:click=move |_| set_zoom_level.update(|z| *z = (*z - 0.5).max(1.0))><ZoomOut size={14} /></button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        _ => {
                            let file_input_ref = NodeRef::<leptos::html::Input>::new();
                            view! {
                                <div class="drop-zone-outer" on:click=move |_| {
                                    if let Some(input) = file_input_ref.get() { let _ = input.click(); }
                                }>
                                    <div class="drop-zone-inner">
                                        <div class="dz-icon-wrap"><ImageIcon size={32} /></div>
                                        <h2 class="dz-title">{crate::text::TXT.editor_empty_title}</h2>
                                        <p class="dz-sub">{crate::text::TXT.editor_empty_desc}</p>
                                        <input type="file" accept="image/*" style="display: none;" on:change=on_file_input node_ref=file_input_ref />
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }}

                <Show when=move || is_dragging.get()>
                    <div class="drag-overlay">
                        <div class="drag-box"><Download size={32} /><span>"Drop to import asset"</span></div>
                    </div>
                </Show>
            </div>

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
                                            <span class="p-tele-label">"TASK"</span>
                                            <span class="p-tele-val">{move || job.map(|id| id.to_string().chars().take(8).collect::<String>()).unwrap_or_else(|| "---".to_string())}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="sidebar-inner">
                                <div class="sb-scroll">
                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag"><Target size={10} /><span>"RESOLUTION"</span></div>
                                            <div class="res-grid">
                                                <div class:active=move || gs.scale.get() == "2x" class="pack-item" on:click=move |_| gs.set_scale.set("2x".to_string())>
                                                    <span class="res-big-num">"2x"</span><span class="pack-price">"2 CR"</span>
                                                </div>
                                                <div class:active=move || gs.scale.get() == "4x" class="pack-item" on:click=move |_| gs.set_scale.set("4x".to_string())>
                                                    <span class="res-big-num">"4x"</span><span class="pack-price">"4 CR"</span>
                                                </div>
                                                <div class:active=move || gs.scale.get() == "6x" class="pack-item" on:click=move |_| gs.set_scale.set("6x".to_string())>
                                                    <span class="res-big-num">"6x"</span><span class="pack-price">"6 CR"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag"><Zap size={10} /><span>"PASSES"</span></div>
                                            
                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <label class="sb-label" style="display: flex; align-items: center;">"AI ARTIFACT REMOVAL"<span title="Pre-processing pass to remove JPEG noise and artifacts. (+1 Credit)" style="cursor: help; margin-left: 4px;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button class:active=move || gs.pre_process_pass.get() on:click=move |_| gs.set_pre_process_pass.set(true)>"On"</button>
                                                    <button class:active=move || !gs.pre_process_pass.get() on:click=move |_| gs.set_pre_process_pass.set(false)>"Off"</button>
                                                </div>
                                            </div>

                                            <div class="sb-field" style="margin-bottom: var(--s-6);">
                                                <label class="sb-label" style="display: flex; align-items: center;">"AI PHOTO RESTORATION"<span title="Rebuilds detail in low-quality or damaged photos. (+1 Credit)" style="cursor: help; margin-left: 4px;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button class:active=move || gs.restoration_pass.get() on:click=move |_| gs.set_restoration_pass.set(true)>"On"</button>
                                                    <button class:active=move || !gs.restoration_pass.get() on:click=move |_| gs.set_restoration_pass.set(false)>"Off"</button>
                                                </div>
                                            </div>

                                            <div class="sb-field">
                                                <label class="sb-label" style="display: flex; align-items: center;">"FACE ENHANCEMENT"<span title="Deeply reconstructs blurry or pixelated faces. (+1 Credit)" style="cursor: help; margin-left: 4px;"><Info size={12} /></span></label>
                                                <div class="seg-control">
                                                    <button class:active=move || gs.face_enhancement.get() on:click=move |_| gs.set_face_enhancement.set(true)>"On"</button>
                                                    <button class:active=move || !gs.face_enhancement.get() on:click=move |_| gs.set_face_enhancement.set(false)>"Off"</button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="card editor-card">
                                        <div class="editor-card-body">
                                            <div class="card-tag clickable" on:click=move |_| gs.set_debug_gemini_only.update(|v| *v = !*v)>
                                                <div style="display: flex; align-items: center; gap: 4px;"><Settings size={10} /><span>"ADVANCED"</span></div>
                                                <div class:rotate-180=move || gs.debug_gemini_only.get()><ChevronDown size={12} /></div>
                                            </div>

                                            <Show when=move || gs.debug_gemini_only.get()>
                                                <div class="sb-field animate-in" style="margin-top: var(--s-6); margin-bottom: var(--s-4);">
                                                    <label class="sb-label"><span>"NOISE REDUCTION"</span><span class="sb-label-val">{move || gs.noise_reduction.get()}</span></label>
                                                    <input type="range" min="0" max="100" class="sb-slider" on:input=move |ev| gs.set_noise_reduction.set(event_target_value(&ev).parse().unwrap_or(0)) prop:value=move || gs.noise_reduction.get() />
                                                </div>
                                                <div class="sb-field animate-in" style="margin-bottom: var(--s-4);">
                                                    <label class="sb-label"><span>"SHARPENING"</span><span class="sb-label-val">{move || gs.sharpen.get()}</span></label>
                                                    <input type="range" min="0" max="100" class="sb-slider" on:input=move |ev| gs.set_sharpen.set(event_target_value(&ev).parse().unwrap_or(0)) prop:value=move || gs.sharpen.get() />
                                                </div>
                                                <div class="sb-field animate-in" style="margin-bottom: var(--s-4);">
                                                    <label class="sb-label"><span>"RESTORATION STRENGTH"</span><span class="sb-label-val">{move || format!("{:.0}%", gs.creativity.get() * 100.0)}</span></label>
                                                    <input type="range" min="0.0" max="1.0" step="0.01" class="sb-slider" on:input=move |ev| gs.set_creativity.set(event_target_value(&ev).parse().unwrap_or(0.35)) prop:value=move || gs.creativity.get() />
                                                </div>
                                            </Show>
                                        </div>
                                    </div>
                                </div>

                                <div class="sb-footer">
                                    <button class="btn btn-primary sb-cta" disabled=move || gs.temp_file.get().is_none() || gs.is_submitting.get() on:click=handle_upscale>
                                        <div class="sb-cta-inner"><Zap size={16} /><span>"UPSCALE ASSET"</span></div>
                                        <div class="sb-cta-badge">{move || cost()}<span style="font-size:0.5rem; opacity:0.6; margin-left:2px;">"CR"</span></div>
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

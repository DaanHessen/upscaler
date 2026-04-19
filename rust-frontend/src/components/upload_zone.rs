use leptos::prelude::*;
use leptos::either::Either;
use lucide_leptos::{UploadCloud, Loader2, Sparkles, Sliders, Image as ImageIcon, ChevronRight};
use crate::auth::use_auth;
use crate::api::{ApiClient, PollResponse};
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth();
    
    // Form State
    let (file, set_file) = signal(Option::<web_sys::File>::None);
    let (preview_url, set_preview_url) = signal(Option::<String>::None);
    let (quality, set_quality) = signal("2K".to_string());
    let (style, set_style) = signal("AUTO".to_string());
    let (temperature, set_temperature) = signal(0.0f32);
    
    // Status State
    let (job_id, set_job_id) = signal(Option::<Uuid>::None);
    let (is_uploading, set_is_uploading) = signal(false);
    let (poll_status, set_poll_status) = signal(Option::<String>::None);
    let (result_url, set_result_url) = signal(Option::<String>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Polling Effect
    Effect::new(move |_| {
        if let Some(id) = job_id.get() {
            let token = auth.session.get().map(|s| s.access_token);
            
            spawn_local(async move {
                loop {
                    match ApiClient::poll_job(id, token.as_deref()).await {
                        Ok(resp) => {
                            set_poll_status.set(Some(resp.status.clone()));
                            if resp.status == "COMPLETED" {
                                set_result_url.set(resp.image_url);
                                set_job_id.set(None);
                                break;
                            } else if resp.status == "FAILED" {
                                set_error_msg.set(resp.error);
                                set_job_id.set(None);
                                break;
                            }
                        }
                        Err(e) => {
                            set_error_msg.set(Some(e));
                            set_job_id.set(None);
                            break;
                        }
                    }
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }
            });
        }
    });

    let on_submit = move |_| {
        if let Some(f) = file.get() {
            set_is_uploading.set(true);
            set_error_msg.set(None);
            set_result_url.set(None);
            
            let q = quality.get();
            let s = style.get();
            let t = temperature.get();
            let token = auth.session.get().map(|s| s.access_token);

            spawn_local(async move {
                match ApiClient::submit_upscale(&f, &q, &s, t, token.as_deref()).await {
                    Ok(resp) => {
                        set_job_id.set(Some(resp.job_id));
                        set_is_uploading.set(false);
                    }
                    Err(e) => {
                        set_error_msg.set(Some(e));
                        set_is_uploading.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="workspace-container fade-in">
            <div class="workspace-header">
                <div class="workflow-stepper">
                    <span class="step active">"1. Source Asset"</span>
                    <ChevronRight size=16 class="step-divider" />
                    <span class=move || if file.get().is_some() { "step active" } else { "step" }>
                        "2. Parameters"
                    </span>
                    <ChevronRight size=16 class="step-divider" />
                    <span class=move || if job_id.get().is_some() || result_url.get().is_some() { "step active" } else { "step" }>
                        "3. Enhancement"
                    </span>
                </div>
            </div>

            <div class="workspace-content">
                {move || match (result_url.get(), poll_status.get()) {
                    (Some(url), _) => Either::Left(view! {
                        <div class="card result-workspace fade-in">
                            <div class="card-header">
                                <h3>"Processing Complete"</h3>
                                <div class="header-actions">
                                    <button class="btn btn-secondary btn-sm" on:click=move |_| {
                                        set_result_url.set(None);
                                        set_file.set(None);
                                        set_preview_url.set(None);
                                    }>"New Project"</button>
                                    <a href=url.clone() target="_blank" class="btn btn-primary btn-sm">"Download Assets"</a>
                                </div>
                            </div>
                            <div class="result-stage">
                                <img src=url />
                            </div>
                        </div>
                    }),
                    (_, Some(status)) => Either::Right(Either::Left(view! {
                        <div class="card status-workspace">
                            <div class="processing-loader">
                                <Loader2 class="animate-spin" size=48 />
                                <h2>"Executing Pipeline"</h2>
                                <p class="text-muted">{move || format!("Status: {}", status)}</p>
                                <div class="progress-bar-container">
                                    <div class="progress-bar-indefinite"></div>
                                </div>
                                <p class="disclaimer">"Securely processing via neural enhancement models. Please remain on this page."</p>
                            </div>
                        </div>
                    })),
                    _ => Either::Right(Either::Right(view! {
                        <div class="dashboard-layout">
                            <div class="card upload-card">
                                <FileUploadArea set_file=set_file set_preview_url=set_preview_url preview_url=preview_url />
                            </div>

                            <div class="card control-sidebar">
                                <div class="sidebar-section">
                                    <label class="section-label">"Output Specifications"</label>
                                    <div class="input-group">
                                        <label>"Resolution"</label>
                                        <select on:change=move |ev| set_quality.set(event_target_value(&ev)) prop:value=quality>
                                            <option value="2K">"High Definition (2K)"</option>
                                            <option value="4K">"Ultra High Definition (4K)"</option>
                                        </select>
                                    </div>
                                </div>

                                <div class="sidebar-section" style="margin-top: 1.5rem;">
                                    <label class="section-label">"Processing Logic"</label>
                                    <div class="input-group">
                                        <label>"Model Context"</label>
                                        <select on:change=move |ev| set_style.set(event_target_value(&ev)) prop:value=style>
                                            <option value="AUTO">"Auto-Detect"</option>
                                            <option value="PHOTOGRAPHY">"Realistic Retouching"</option>
                                            <option value="ILLUSTRATION">"Vector Reconstruction"</option>
                                        </select>
                                    </div>
                                    
                                    <div class="input-group" style="margin-top: 1.25rem;">
                                        <div style="display: flex; justify-content: space-between">
                                            <label>"Reconstruction Depth"</label>
                                            <span class="value-tag">{move || format!("{:.1}", temperature.get())}</span>
                                        </div>
                                        <input 
                                            type="range" min="0.0" max="2.0" step="0.1"
                                            on:input=move |ev| if let Ok(v) = event_target_value(&ev).parse::<f32>() { set_temperature.set(v); }
                                            prop:value=move || temperature.get().to_string()
                                        />
                                    </div>
                                </div>

                                <div class="sidebar-footer">
                                    {move || match error_msg.get() {
                                        Some(msg) => Either::Left(view! {
                                            <p class="error-text" style="margin-bottom: 1rem;">{msg}</p>
                                        }),
                                        None => Either::Right(()),
                                    }}
                                    <button 
                                        class="btn btn-primary" style="width: 100%;"
                                        on:click=on_submit
                                        disabled=move || file.get().is_none() || is_uploading.get()
                                    >
                                        <Sparkles size=18 />
                                        {move || if is_uploading.get() { "Initializing..." } else { "Execute Project" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }))
                }}
            </div>

            <style>
                ".workspace-container { max-width: 1100px; margin: 0 auto; }
                .workspace-header { margin-bottom: 2rem; display: flex; justify-content: center; }
                
                .workflow-stepper { display: flex; align-items: center; gap: 1rem; }
                .step { font-size: 0.75rem; font-weight: 600; text-transform: uppercase; color: var(--text-muted); opacity: 0.5; transition: all 0.3s; }
                .step.active { opacity: 1; color: var(--primary); }
                .step-divider { color: var(--border-color); }

                .dashboard-layout { display: grid; grid-template-columns: 1fr 320px; gap: 2rem; align-items: start; }
                
                .upload-card { padding: 0.5rem; height: 500px; }
                .control-sidebar { padding: 2rem; position: sticky; top: 100px; }
                
                .section-label { display: block; font-size: 0.7rem; font-weight: 800; color: var(--text-color); margin-bottom: 1rem; text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.7; }
                .value-tag { font-family: monospace; font-size: 0.875rem; color: var(--primary); }

                .status-workspace { padding: 6rem 2rem; text-align: center; }
                .processing-loader { display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
                .progress-bar-container { width: 100%; max-width: 300px; height: 4px; background: var(--surface-lighter); border-radius: 100px; overflow: hidden; }
                .progress-bar-indefinite { width: 40%; height: 100%; background: var(--primary); border-radius: 100px; animation: progressIndefinite 2s infinite ease-in-out; }
                @keyframes progressIndefinite { 0% { transform: translateX(-100%); } 100% { transform: translateX(300%); } }
                .disclaimer { font-size: 0.75rem; color: var(--text-muted); max-width: 400px; }

                .result-workspace { overflow: hidden; }
                .card-header { padding: 1.5rem 2rem; border-bottom: 1px solid var(--border-color); display: flex; justify-content: space-between; align-items: center; }
                .result-stage { padding: 2rem; background: #000; display: flex; justify-content: center; align-items: center; min-height: 400px; }
                .result-stage img { max-width: 100%; max-height: 70vh; box-shadow: 0 20px 50px rgba(0,0,0,0.5); }

                .animate-spin { animation: spin 1s linear infinite; }
                @keyframes spin { 100% { transform: rotate(360deg); } }

                @media (max-width: 900px) {
                    .dashboard-layout { grid-template-columns: 1fr; }
                    .control-sidebar { position: static; }
                }
                "
            </style>
        </div>
    }
}

#[component]
fn FileUploadArea(
    set_file: WriteSignal<Option<web_sys::File>>,
    set_preview_url: WriteSignal<Option<String>>,
    preview_url: ReadSignal<Option<String>>
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = signal(false);

    let on_file_change = move |ev: web_sys::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            if let Some(f) = files.item(0) {
                set_file.set(Some(f.clone()));
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&f) {
                    set_preview_url.set(Some(url));
                }
            }
        }
    };

    view! {
        <div 
            class=move || format!("dropzone-v2 {}", if is_dragging.get() { "dragging" } else { "" })
            on:dragover=move |ev: web_sys::DragEvent| { ev.prevent_default(); set_is_dragging.set(true); }
            on:dragleave=move |_| set_is_dragging.set(false)
            on:drop=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                set_is_dragging.set(false);
                if let Some(data) = ev.data_transfer() {
                    let files: Option<web_sys::FileList> = data.files();
                    if let Some(files) = files {
                        if let Some(f) = files.item(0) {
                            let file: web_sys::File = f.clone();
                            set_file.set(Some(file.clone()));
                            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                                set_preview_url.set(Some(url));
                            }
                        }
                    }
                }
            }
            on:click=move |_| {
                let doc = web_sys::window().unwrap().document().unwrap();
                if let Some(btn) = doc.get_element_by_id("prof-file-upload") {
                    let input = btn.unchecked_into::<web_sys::HtmlInputElement>();
                    input.click();
                }
            }
        >
            <input type="file" id="prof-file-upload" style="display: none" on:change=on_file_change accept="image/*" />
            
            {move || match preview_url.get() {
                Some(url) => Either::Left(view! {
                    <div class="preview-container fade-in">
                        <img src=url />
                        <div class="change-overlay">"Update Source Asset"</div>
                    </div>
                }),
                None => Either::Right(view! {
                    <div class="empty-upload">
                        <div class="icon-circle">
                            <UploadCloud size=32 />
                        </div>
                        <h3>"Direct Asset Upload"</h3>
                        <p>"Securely upload high-resolution images for enhancement pipeline."</p>
                        <ul class="upload-specs">
                            <li>"Supported: JPEG, PNG, WEBP"</li>
                            <li>"Max Payload: 10MB"</li>
                        </ul>
                        <button class="btn btn-secondary">"Select File"</button>
                    </div>
                })
            }}
        </div>

        <style>
            ".dropzone-v2 {
                width: 100%;
                height: 100%;
                border-radius: 8px;
                display: flex;
                align-items: center;
                justify-content: center;
                cursor: pointer;
                transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                background: #05070a;
                border: 2px dashed var(--border-color);
                position: relative;
                overflow: hidden;
            }
            .dropzone-v2:hover { border-color: var(--primary); background: #080a0f; }
            .dropzone-v2.dragging { border-color: var(--primary); background: rgba(59, 130, 246, 0.05); }
            
            .empty-upload { text-align: center; padding: 3rem; display: flex; flex-direction: column; align-items: center; gap: 1rem; }
            .icon-circle { width: 64px; height: 64px; border-radius: 50%; background: var(--surface-lighter); display: flex; align-items: center; justify-content: center; color: var(--primary); margin-bottom: 0.5rem; }
            .empty-upload h3 { font-size: 1.25rem; font-weight: 700; }
            .empty-upload p { font-size: 0.875rem; color: var(--text-muted); max-width: 250px; }
            .upload-specs { list-style: none; display: flex; flex-direction: column; gap: 0.5rem; margin: 1rem 0; }
            .upload-specs li { font-size: 0.7rem; font-weight: 700; background: var(--surface-lighter); padding: 4px 12px; border-radius: 4px; color: var(--text-muted); text-transform: uppercase; }

            .preview-container { width: 100%; height: 100%; position: relative; display: flex; align-items: center; justify-content: center; background: #000; }
            .preview-container img { max-width: 100%; max-height: 100%; object-fit: contain; }
            .change-overlay { position: absolute; inset: 0; background: rgba(0,0,0,0.6); opacity: 0; display: flex; align-items: center; justify-content: center; font-size: 0.875rem; font-weight: 600; transition: opacity 0.2s; }
            .preview-container:hover .change-overlay { opacity: 1; }
            "
        </style>
    }
}

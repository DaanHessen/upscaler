use leptos::prelude::*;
use leptos::either::Either;
use leptos::task::spawn_local;
use crate::components::icons::{Upload, Download, RefreshCw, AlertCircle, ImageIcon, Zap};
use crate::auth::use_auth;
use crate::api::{ApiClient, ModerateResponse};

#[derive(Clone, Debug, PartialEq)]
enum DashboardState {
    Setup,
    Detecting(web_sys::File),
    Review(web_sys::File, ModerateResponse),
    Processing(String), // job_id
    Result(String),     // result_url
    Error(String),
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (state, set_state) = signal(DashboardState::Setup);
    
    view! {
        <div class="dashboard-container fade-in">
            {move || match state.get() {
                DashboardState::Setup => Either::Left(view! {
                    <SetupStage 
                        on_file_select=move |file| set_state.set(DashboardState::Detecting(file)) 
                    />
                }),
                DashboardState::Detecting(file) => {
                    let file_stored = StoredValue::new(file.clone());
                    Either::Right(Either::Left(view! {
                        <DetectingStage 
                            file=file
                            on_detected=move |res| set_state.set(DashboardState::Review(file_stored.get_value(), res)) 
                            on_error=move |err| set_state.set(DashboardState::Error(err)) 
                        />
                    }))
                },
                DashboardState::Review(file, detection) => {
                    Either::Right(Either::Right(Either::Left(view! {
                        <ReviewStage 
                            file=file
                            detection=detection 
                            on_start=move |job_id| set_state.set(DashboardState::Processing(job_id))
                            on_cancel=move |_| set_state.set(DashboardState::Setup)
                        />
                    })))
                },
                DashboardState::Processing(job_id) => Either::Right(Either::Right(Either::Right(Either::Left(view! {
                    <ProcessingStage job_id=job_id on_complete=move |url| set_state.set(DashboardState::Result(url)) on_error=move |err| set_state.set(DashboardState::Error(err)) />
                })))),
                DashboardState::Result(url) => Either::Right(Either::Right(Either::Right(Either::Right(Either::Left(view! {
                    <ResultStage result_url=url on_reset=move |_| set_state.set(DashboardState::Setup) />
                }))))),
                DashboardState::Error(err) => Either::Right(Either::Right(Either::Right(Either::Right(Either::Right(view! {
                    <ErrorStage message=err on_retry=move |_| set_state.set(DashboardState::Setup) />
                }))))),
            }}
        </div>
    }
}

#[component]
fn SetupStage<F>(on_file_select: F) -> impl IntoView 
where F: Fn(web_sys::File) + 'static + Copy {
    view! {
        <div class="stage-wrapper">
            <div class="stage-header">
                <h2>"Professional Upscale"</h2>
                <p class="muted">"Upload an image to begin the super-resolution pipeline."</p>
            </div>

            <div class="card setup-card single-card">
                <div class="card-header">
                    <Upload size=18 />
                    <span>"Source Asset"</span>
                </div>
                <FileUploadArea on_file_select=on_file_select />
            </div>

            <div class="stage-footer">
                <div class="technical-limits">
                    <span class="limit-item">"MAX PAYLOAD: 25MB"</span>
                    <span class="limit-item">"FORMATS: JPG, PNG, WEBP"</span>
                </div>
            </div>
            
            <style>
                ".stage-wrapper { max-width: 900px; margin: 0 auto; padding: 2rem 0; }
                .stage-header { margin-bottom: 4rem; }
                .single-card { margin-bottom: 2rem; }
                "
            </style>
        </div>
    }
}

#[component]
fn DetectingStage<FD, FE>(file: web_sys::File, on_detected: FD, on_error: FE) -> impl IntoView 
where FD: Fn(ModerateResponse) + 'static + Copy, FE: Fn(String) + 'static + Copy {
    let auth = use_auth();
    
    spawn_local(async move {
        let token = auth.session.get().map(|s| s.access_token);
        match ApiClient::moderate(&file, token.as_deref()).await {
            Ok(res) => {
                if res.nsfw {
                    on_error("Content violates safety guidelines (NSFW detected).".to_string());
                } else {
                    on_detected(res);
                }
            }
            Err(e) => on_error(format!("Detection Failed: {}", e)),
        }
    });

    view! {
        <div class="stage-wrapper fade-in" style="text-align: center; padding: 8rem 0;">
            <div class="processing-visual">
                <div class="outer-ring"></div>
                <div class="icon-stage"><RefreshCw size={32} /></div>
            </div>
            <h2 style="margin-top: 2rem; font-family: var(--font-mono); font-size: 1rem; letter-spacing: 0.05em;">"ANALYZING SIGNAL..."</h2>
            <p class="muted" style="margin-top: 0.5rem; font-size: 0.8rem;">"Executing local inference for optimal reconstruction path."</p>
        </div>
    }
}

#[component]
fn ReviewStage<FS, FC>(file: web_sys::File, detection: ModerateResponse, on_start: FS, on_cancel: FC) -> impl IntoView 
where FS: Fn(String) + 'static + Copy, FC: Fn(web_sys::MouseEvent) + 'static + Copy {
    let auth = use_auth();
    let (quality, set_quality) = signal("2K".to_string());
    let (style, set_style) = signal(detection.detected_style.clone());
    let (temperature, set_temperature) = signal(0.0f32);
    let (loading, set_loading) = signal(false);

    let file_for_upload = file.clone();
    let handle_upscale = move |_| {
        set_loading.set(true);
        let token = auth.session.get().map(|s| s.access_token);
        let q_val = quality.get();
        let s_val = style.get();
        let t_val = temperature.get();
        let f_clone = file_for_upload.clone();
        
        spawn_local(async move {
            match ApiClient::submit_upscale(&f_clone, &q_val, &s_val, t_val, token.as_deref()).await {
                Ok(resp) => on_start(resp.job_id.to_string()),
                Err(e) => {
                    leptos::logging::error!("Upscale failed: {}", e);
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="stage-wrapper fade-in">
            <div class="stage-header">
                <h2>"Configuration"</h2>
                <p class="muted">"Verified asset. Adjust parameters for high-fidelity reconstruction."</p>
            </div>

            <div class="setup-grid">
                <div class="card preview-card">
                    <div class="card-header">
                        <ImageIcon size={18} />
                        <span>"Detected Signal"</span>
                    </div>
                    <div class="detection-badge-container">
                        <div class="style-badge">
                            <Zap size={14} />
                            <span>{move || style.get()}</span>
                        </div>
                    </div>
                    <div class="preview-visual">
                        <img src=web_sys::Url::create_object_url_with_blob(&file).unwrap() />
                    </div>
                </div>

                <div class="card params-card">
                    <div class="card-header">
                        <Zap size=18 />
                        <span>"Parameters"</span>
                    </div>
                    <div class="params-content">
                        <div class="param-group">
                            <label>"Target Resolution"</label>
                            <select on:change=move |ev| set_quality.set(event_target_value(&ev))>
                                <option value="2K">"2K (High Fidelity)"</option>
                                <option value="4K">"4K (Ultra High)"</option>
                            </select>
                            <span class="param-hint">
                                {move || if quality.get() == "2K" { "Consumes 2 Credits" } else { "Consumes 4 Credits" }}
                            </span>
                        </div>

                        <div class="param-group">
                            <label>"Style Mode"</label>
                            <select on:change=move |ev| set_style.set(event_target_value(&ev)) prop:value=style>
                                <option value="PHOTOGRAPHY">"Photography"</option>
                                <option value="ILLUSTRATION">"Illustration"</option>
                            </select>
                            <span class="param-hint">"Detection override"</span>
                        </div>

                        <div class="param-group">
                            <label>"Creativity (Temp): " {move || format!("{:.1}", temperature.get())}</label>
                            <input 
                                type="range" 
                                min="0.0" 
                                max="2.0" 
                                step="0.1" 
                                prop:value=move || temperature.get().to_string()
                                on:input=move |ev| set_temperature.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            />
                            <span class="param-hint">"Lower = More Faithful"</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="stage-footer">
                <button class="btn btn-secondary" on:click=on_cancel>"CANCEL"</button>
                <button 
                    class="btn btn-primary btn-lg" 
                    disabled=loading
                    on:click=handle_upscale
                >
                    {move || if loading.get() { "Enqueuing..." } else { "Begin Reconstruction" }}
                </button>
            </div>

            <style>
                ".detection-badge-container { padding: 1rem 1.5rem 0; }
                .style-badge { display: inline-flex; align-items: center; gap: 0.5rem; background: rgba(88, 166, 255, 0.1); color: var(--accent); padding: 0.4rem 0.75rem; border-radius: 4px; border: 1px solid rgba(88, 166, 255, 0.2); font-size: 0.75rem; font-weight: 700; font-family: var(--font-mono); text-transform: uppercase; }
                .preview-visual { padding: 1.5rem; display: flex; align-items: center; justify-content: center; }
                .preview-visual img { max-width: 100%; max-height: 240px; border-radius: 4px; border: 1px solid var(--border-color); }
                
                .params-card { height: 100%; }
                .setup-grid { display: grid; grid-template-columns: 1fr 340px; gap: 2rem; margin-bottom: 3rem; align-items: stretch; }
                
                input[type=range] { width: 100%; }
                "
            </style>
        </div>
    }
}

#[component]
fn FileUploadArea<F>(on_file_select: F) -> impl IntoView 
where F: Fn(web_sys::File) + 'static + Copy {
    let (is_over, set_is_over) = signal(false);

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_is_over.set(false);
        if let Some(dt) = ev.data_transfer() {
            if let Some(files) = dt.files() {
                if let Some(f) = files.get(0) {
                    on_file_select(f);
                }
            }
        }
    };

    let on_input = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(f) = files.get(0) {
                on_file_select(f);
            }
        }
    };

    view! {
        <div 
            class=move || if is_over.get() { "upload-dropzone drag-over" } else { "upload-dropzone" }
            on:dragover=move |ev| { ev.prevent_default(); set_is_over.set(true); }
            on:dragleave=move |_| set_is_over.set(false)
            on:drop=on_drop
        >
            <input type="file" id="file-upload" on:change=on_input style="display: none;" accept="image/*" />
            
            <label for="file-upload" class="dropzone-content">
                <div class="icon-circle"><ImageIcon size={24} /></div>
                <div class="text-content">
                    <h3>"Select source image"</h3>
                    <p>"or drag and drop into this area"</p>
                </div>
            </label>

            <style>
                ".upload-dropzone { min-height: 300px; display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; cursor: pointer; transition: all 0.2s; background: var(--bg-color); border: 1px dashed var(--border-color); border-radius: 8px; margin: 1.5rem; }
                .upload-dropzone:hover { border-color: var(--accent); background: var(--surface-color); }
                .upload-dropzone.drag-over { background: rgba(88, 166, 255, 0.05); border-color: var(--accent); }
                
                .dropzone-content { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1rem; width: 100%; height: 100%; justify-content: center; padding: 2rem; cursor: pointer; }
                .icon-circle { width: 48px; height: 48px; border-radius: 50%; background: var(--surface-color); display: flex; align-items: center; justify-content: center; color: var(--text-muted); border: 1px solid var(--border-color); }
                
                .text-content h3 { font-size: 0.9rem; font-weight: 600; margin-bottom: 0.25rem; }
                .text-content p { font-size: 0.8rem; color: var(--text-muted); }
                "
            </style>
        </div>
    }
}

#[component]
fn ProcessingStage<FC, FE>(job_id: String, on_complete: FC, on_error: FE) -> impl IntoView 
where FC: Fn(String) + 'static + Copy, FE: Fn(String) + 'static + Copy {
    let auth = use_auth();
    let (status, set_status) = signal("Initializing Pipeline...".to_string());
    let (progress, set_progress) = signal(0);

    let job_id_cloned = job_id.clone();
    spawn_local(async move {
        let token = auth.session.get().map(|s| s.access_token);
        let mut attempts = 0;
        
        loop {
            if let Ok(job_uuid) = uuid::Uuid::parse_str(&job_id_cloned) {
                match ApiClient::poll_job(job_uuid, token.as_deref()).await {
                    Ok(res) => {
                        let backend_status = res.status.clone();
                        let user_status = match backend_status.as_str() {
                            "PENDING" => "Waiting for Pipeline Slot...",
                            "PROCESSING" => {
                                if attempts < 5 {
                                    "Quantizing frequency layers..."
                                } else if attempts < 15 {
                                    "Reconstructing high-frequency details..."
                                } else {
                                    "Finalizing reconstruction..."
                                }
                            },
                            "COMPLETED" => "Verified.",
                            _ => "Processing...",
                        };
                        set_status.set(user_status.to_string());
                        
                        match backend_status.as_str() {
                            "COMPLETED" => {
                                if let Some(url) = res.image_url {
                                    on_complete(url);
                                    break;
                                }
                            }
                            "FAILED" => {
                                on_error("Upscale Error: Neural engine failed to converge.".to_string());
                                break;
                            }
                            _ => {
                                set_progress.update(|p| if *p < 95 { *p += 3 });
                            }
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("Status poll failed: {}", e);
                    }
                }
            }
            
            attempts += 1;
            if attempts > 60 { 
                on_error("Pipeline Timeout: Maximum duration exceeded.".to_string());
                break;
            }
            
            gloo_timers::future::TimeoutFuture::new(2000).await;
        }
    });

    view! {
        <div class="stage-wrapper fade-in" style="text-align: center; padding: 10rem 0;">
            <div class="processing-visual">
                <div class="outer-ring"></div>
                <div class="inner-ring"></div>
                <div class="icon-stage"><RefreshCw size={48} /></div>
            </div>
            
            <h2 style="margin-top: 3rem; font-family: var(--font-mono); font-size: 1rem;">{move || status.get()}</h2>
            <p class="muted" style="margin-top: 0.5rem; font-size: 0.8rem;">"Executing neural inference at target resolution."</p>
            
            <div class="progress-box">
                <div class="progress-bar" style=move || format!("width: {}%", progress.get())></div>
            </div>
            
            <style>
                ".processing-visual { position: relative; width: 100px; height: 100px; margin: 0 auto; display: flex; align-items: center; justify-content: center; }
                .outer-ring { position: absolute; width: 100%; height: 100%; border: 1px solid var(--border-color); border-radius: 50%; border-top-color: var(--accent); animation: spin 2s linear infinite; }
                .inner-ring { position: absolute; width: 70%; height: 70%; border: 1px solid var(--border-color); border-radius: 50%; border-bottom-color: var(--accent); animation: spin-reverse 1.5s linear infinite; }
                .icon-stage { color: var(--accent); opacity: 0.5; }
                
                @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
                @keyframes spin-reverse { from { transform: rotate(360deg); } to { transform: rotate(0deg); } }

                .progress-box { width: 300px; height: 2px; background: var(--surface-lighter); border-radius: 2px; margin: 3rem auto 0; overflow: hidden; }
                .progress-bar { height: 100%; background: var(--accent); transition: width 0.5s ease-out; }
                "
            </style>
        </div>
    }
}

#[component]
fn ResultStage<FR>(result_url: String, on_reset: FR) -> impl IntoView 
where FR: Fn(web_sys::MouseEvent) + 'static + Copy {
    view! {
        <div class="stage-wrapper fade-in">
            <div class="stage-header">
                <h2>"Success"</h2>
                <p class="muted">"Asset reconstruction verified. Infrastructure is ready for export."</p>
            </div>

            <div class="result-card card">
                <div class="result-visual">
                    <img src=result_url.clone() />
                </div>
                
                <div class="result-footer">
                    <div class="result-meta">
                        <span class="label">"PIPELINE STATUS"</span>
                        <span class="value success">"VERIFIED"</span>
                    </div>
                    <div class="result-actions">
                        <button class="btn btn-secondary" on:click=on_reset>"NEW PROJECT"</button>
                        <a href=result_url target="_blank" class="btn btn-primary" style="text-decoration: none;">
                            <Download size={16} />
                            "EXPORT ASSET"
                        </a>
                    </div>
                </div>
            </div>

            <style>
                ".result-visual { min-height: 400px; background: #000; display: flex; align-items: center; justify-content: center; border-bottom: 1px solid var(--border-color); }
                .result-visual img { max-width: 100%; max-height: 600px; }
                
                .result-footer { padding: 1.5rem 2rem; display: flex; justify-content: space-between; align-items: center; }
                .result-meta { display: flex; flex-direction: column; gap: 0.25rem; }
                .result-meta .label { font-size: 0.65rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
                .result-meta .value.success { color: var(--success); font-weight: 700; font-size: 0.8rem; font-family: var(--font-mono); }
                
                .result-actions { display: flex; gap: 0.75rem; }
                "
            </style>
        </div>
    }
}

#[component]
fn ErrorStage<FR>(message: String, on_retry: FR) -> impl IntoView 
where FR: Fn(web_sys::MouseEvent) + 'static + Copy {
    view! {
        <div class="stage-wrapper fade-in" style="text-align: center; padding: 10rem 0;">
            <div class="icon-circle" style="background: rgba(248, 81, 73, 0.1); color: var(--error); margin: 0 auto; width: 64px; height: 64px; border-radius: 50%; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(248, 81, 73, 0.2);">
                <AlertCircle size={32} />
            </div>
            <h2 style="margin-top: 2rem; font-size: 1.5rem;">"Pipeline Error"</h2>
            <p class="muted" style="margin-top: 0.75rem; max-width: 400px; margin: 0.75rem auto 3rem;">{message}</p>
            
            <button class="btn btn-primary" on:click=on_retry>"RETURN TO WORKSPACE"</button>
        </div>
    }
}

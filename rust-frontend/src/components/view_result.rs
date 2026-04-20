use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_navigate};
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::components::icons::{Download, RefreshCw, AlertCircle, Settings};
use crate::components::comparison_slider::ComparisonSlider;

#[component]
pub fn ViewResult() -> impl IntoView {
    let params = use_params_map();
    let auth = use_auth();
    
    let job_id = move || params.get().get("job_id").map(|s| s.to_string()).unwrap_or_default();
    
    let job_data = LocalResource::new(move || {
        let id = job_id();
        let token = auth.session.get().map(|s| s.access_token);
        async move {
            if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
                ApiClient::poll_job(uuid, token.as_deref()).await
            } else {
                Err("Invalid ID".to_string())
            }
        }
    });

    view! {
        <div class="view-result-page fade-in">
            <Suspense fallback=|| view! { <div class="loading-full-page"><RefreshCw size={32} /></div> }>
                {move || Suspend::new(async move {
                    let res = job_data.await;
                    match res {
                        Ok(data) => {
                            if data.status == "COMPLETED" {
                                view! { <ResultView data=data job_id=job_id() /> }.into_any()
                            } else if data.status == "FAILED" {
                                view! { <ErrorView message=data.error.unwrap_or_else(|| "Unknown engine failure".to_string()) /> }.into_any()
                            } else {
                                view! { <PollingView job_id=job_id() on_complete=move |_| job_data.refetch() /> }.into_any()
                            }
                        }
                        Err(e) => view! { <ErrorView message=e /> }.into_any(),
                    }
                })}
            </Suspense>

            <style>
                ".view-result-page { max-width: 1200px; margin: 0 auto; }
                .loading-full-page { display: flex; align-items: center; justify-content: center; min-height: 400px; color: var(--accent); }
                "
            </style>
        </div>
    }
}

#[component]
fn ResultView(data: crate::api::PollResponse, job_id: String) -> impl IntoView {
    let navigate = use_navigate();
    
    view! {
        <div class="result-container fade-in">
            <div class="page-header">
                <div class="header-main">
                    <h1>"Reconstruction Verified"</h1>
                    <p class="muted">"Neural pipeline has successfully converged at target resolution."</p>
                </div>
                <div class="header-actions">
                    <button class="btn btn-secondary" on:click=move |_| navigate("/", Default::default())>"TRY AGAIN"</button>
                    <button class="btn btn-secondary" disabled=true title="Coming soon">"REQUEST REFUND"</button>
                    <a href=data.image_url.clone().unwrap_or_default() target="_blank" class="btn btn-primary" style="text-decoration: none;">
                        <Download size={16} />
                        "DOWNLOAD ASSET"
                    </a>
                </div>
            </div>

            <div class="result-main">
                <div class="result-slider-box">
                    <ComparisonSlider 
                        before_url="/assets/hero_before.png".to_string() 
                        after_url=data.image_url.unwrap_or_default()
                        before_label="Original Signal"
                        after_label="Upscaled Reconstruction"
                    />
                </div>

                <div class="result-sidebar">
                    <div class="card settings-card">
                        <div class="card-header">
                            <Settings size={18} />
                            <span>"Pipeline Specs"</span>
                        </div>
                        <div class="settings-list">
                            <div class="s-item">
                                <span class="s-label">"JOB IDENTIFIER"</span>
                                <span class="s-value font-mono">{job_id}</span>
                            </div>
                            <div class="s-item">
                                <span class="s-label">"STATUS"</span>
                                <span class="s-value success">"VERIFIED"</span>
                            </div>
                            <div class="s-item">
                                <span class="s-label">"ENGINE"</span>
                                <span class="s-value">"V7.1 STABLE"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".page-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 3rem; }
                .header-actions { display: flex; gap: 0.75rem; }
                
                .result-main { display: grid; grid-template-columns: 1fr 300px; gap: 2rem; align-items: flex-start; }
                .result-slider-box { border-radius: 12px; overflow: hidden; border: 1px solid var(--border-color); }
                
                .settings-list { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; }
                .s-item { display: flex; flex-direction: column; gap: 0.25rem; }
                .s-label { font-size: 0.6rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; text-transform: uppercase; }
                .s-value { font-size: 0.85rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
                .s-value.success { color: var(--success); }
                
                @media (max-width: 1000px) {
                    .result-main { grid-template-columns: 1fr; }
                    .page-header { flex-direction: column; align-items: flex-start; gap: 1.5rem; border-bottom: 1px solid var(--border-color); padding-bottom: 1.5rem; }
                    .header-actions { width: 100%; flex-direction: column; }
                    .header-actions .btn { width: 100%; }
                }
                "
            </style>
        </div>
    }
}

#[component]
fn PollingView<F>(job_id: String, on_complete: F) -> impl IntoView 
where F: Fn(()) + 'static + Copy {
    let auth = use_auth();
    let (status_text, set_status_text) = signal("Initializing Pipeline...".to_string());
    
    leptos::task::spawn_local(async move {
        let token = auth.session.get().map(|s| s.access_token);
        let mut attempts = 0;
        loop {
            if let Ok(uuid) = uuid::Uuid::parse_str(&job_id) {
                match ApiClient::poll_job(uuid, token.as_deref()).await {
                    Ok(res) => {
                        if res.status == "COMPLETED" || res.status == "FAILED" {
                            on_complete(());
                            break;
                        }
                        set_status_text.set(match res.status.as_str() {
                            "PENDING" => "Waiting for Pipeline Slot...".to_string(),
                            "PROCESSING" => "Refining high-frequency details...".to_string(),
                            _ => "Processing...".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            attempts += 1;
            if attempts > 60 { break; }
            gloo_timers::future::TimeoutFuture::new(2000).await;
        }
    });

    view! {
        <div class="polling-container fade-in">
            <div class="processing-visual">
                <div class="outer-ring"></div>
                <div class="inner-icon"><RefreshCw size={48} /></div>
            </div>
            <h2>{move || status_text.get()}</h2>
            <p class="muted">"Executing neural inference at target resolution. 10GB/s I/O in progress."</p>
            
            <style>
                ".polling-container { text-align: center; padding: 10rem 0; display: flex; flex-direction: column; align-items: center; gap: 2rem; }
                .processing-visual { position: relative; width: 100px; height: 100px; display: flex; align-items: center; justify-content: center; }
                .outer-ring { position: absolute; width: 100%; height: 100%; border: 2px solid var(--border-color); border-radius: 50%; border-top-color: var(--accent); animation: spin 1.5s linear infinite; }
                .inner-icon { color: var(--accent); opacity: 0.5; }
                
                @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
                "
            </style>
        </div>
    }
}

#[component]
fn ErrorView(message: String) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class="error-container fade-in">
            <AlertCircle size={64} />
            <h2>"Pipeline Error"</h2>
            <p class="muted">{message}</p>
            <button class="btn btn-primary" on:click=move |_| navigate("/", Default::default())>"RETURN HOME"</button>
            <style>
                ".error-container { text-align: center; padding: 10rem 0; display: flex; flex-direction: column; align-items: center; gap: 2rem; }
                .error-container h2 { font-size: 2rem; }
                .error-container p { max-width: 400px; }
                "
            </style>
        </div>
    }
}

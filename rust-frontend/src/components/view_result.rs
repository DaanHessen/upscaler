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
fn PollingView<F>(job_id: String, on_complete: F) -> impl IntoView 
where F: Fn(()) + 'static + Copy {
    let auth = use_auth();
    let (poll_res, set_poll_res) = signal::<Option<crate::api::PollResponse>>(None);
    
    let job_id_clone = job_id.clone();
    leptos::task::spawn_local(async move {
        let token = auth.session.get().map(|s| s.access_token);
        let mut attempts = 0;
        loop {
            if let Ok(uuid) = uuid::Uuid::parse_str(&job_id_clone) {
                match ApiClient::poll_job(uuid, token.as_deref()).await {
                    Ok(res) => {
                        let status = res.status.clone();
                        set_poll_res.set(Some(res));
                        if status == "COMPLETED" || status == "FAILED" {
                            on_complete(());
                            break;
                        }
                    }
                    _ => {}
                }
            }
            attempts += 1;
            if attempts > 300 { break; } 
            gloo_timers::future::TimeoutFuture::new(2000).await;
        }
    });

    let stage_info = move || {
        if let Some(res) = poll_res.get() {
            match res.status.as_str() {
                "PENDING" => {
                    let pos = res.queue_position.unwrap_or(1);
                    ("IN QUEUE", format!("Position #{} in queue", pos), "Waiting for an available compute node...")
                },
                "PROCESSING" => {
                    ("RECONSTRUCTING", "Engine Engaged...".to_string(), "Gemini Vision is enhancing your asset...")
                },
                _ => ("FINALIZING", "Synchronizing Store...".to_string(), "Finalizing and storing your high-res asset...")
            }
        } else {
            ("STARTING", "Initializing...".to_string(), "Establishing connection to Studio Backend...")
        }
    };

    let display_id = job_id.chars().take(8).collect::<String>();

    view! {
        <div class="polling-container fade-in">
            <div class="processing-studio-card card shadow-lg">
                <div class="params-body">
                    <div class="card-tag">
                        <Settings size={10} />
                        <span>"STUDIO PULSE"</span>
                    </div>

                    <div class="studio-processing-view">
                        <div class="pulse-visual">
                            <div class="pulse-ring"></div>
                            <div class="pulse-ring delay-1"></div>
                            <div class="pulse-core">
                                <RefreshCw size={32} />
                            </div>
                        </div>

                        <div class="processing-meta">
                            <span class="stage-tag">{move || stage_info().0}</span>
                            <h2 class="stage-title">{move || stage_info().1}</h2>
                            <p class="muted stage-desc">{move || stage_info().2}</p>
                        </div>

                        <div class="telemetry-bar">
                            <div class="telemetry-segment">
                                <span class="t-label">"JOB IDENT"</span>
                                <span class="t-value">{display_id}</span>
                            </div>
                            <div class="telemetry-segment">
                                <span class="t-label">"ENGINE"</span>
                                <span class="t-value">"V2 STUDIO"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <style>
                ".polling-container { display: flex; align-items: center; justify-content: center; padding: var(--s-20) 0; min-height: 60vh; }
                .processing-studio-card { width: 100%; max-width: 500px; background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); }
                
                .studio-processing-view { display: flex; flex-direction: column; align-items: center; gap: var(--s-10); padding: var(--s-10) 0; }
                
                .pulse-visual { position: relative; width: 120px; height: 120px; display: flex; align-items: center; justify-content: center; }
                .pulse-core { color: hsl(var(--accent)); animation: spin 4s linear infinite; z-index: 2; background: hsl(var(--bg)); border-radius: 50%; padding: var(--s-4); border: 1px solid var(--glass-border); }
                
                .pulse-ring { position: absolute; width: 100%; height: 100%; border: 1px solid hsl(var(--accent)); border-radius: 50%; animation: pulse-out 3s ease-out infinite; opacity: 0; }
                .pulse-ring.delay-1 { animation-delay: 1.5s; }

                @keyframes pulse-out {
                    0% { transform: scale(0.6); opacity: 0; }
                    50% { opacity: 0.3; }
                    100% { transform: scale(1.4); opacity: 0; }
                }

                .processing-meta { text-align: center; display: flex; flex-direction: column; gap: var(--s-2); }
                .stage-tag { font-size: 0.625rem; font-weight: 900; color: hsl(var(--accent)); letter-spacing: 0.2em; text-transform: uppercase; }
                .stage-title { font-size: 1.5rem; font-weight: 850; letter-spacing: -0.02em; color: hsl(var(--text)); }
                .stage-desc { font-size: 0.8125rem; max-width: 320px; margin: 0 auto; color: hsl(var(--text-dim)); }

                .telemetry-bar { display: flex; gap: var(--s-8); margin-top: var(--s-4); padding-top: var(--s-6); border-top: 1px solid var(--glass-border); width: 100%; justify-content: center; }
                .telemetry-segment { display: flex; flex-direction: column; gap: 2px; }
                .t-label { font-size: 0.5rem; font-weight: 900; color: hsl(var(--text-dim)); opacity: 0.5; letter-spacing: 0.1em; }
                .t-value { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; color: hsl(var(--text-muted)); }

                @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
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
                    <h1 class="text-gradient">"Upscale Complete"</h1>
                    <p class="muted">"Your asset has been successfully reconstructed."</p>
                </div>
                <div class="header-actions">
                    <button class="btn btn-secondary btn-sm" on:click=move |_| navigate("/configure", Default::default())>"NEW UPSCALE"</button>
                    <a href=data.image_url.clone().unwrap_or_default() target="_blank" class="btn btn-primary btn-sm" style="text-decoration: none;">
                        <Download size={14} />
                        "DOWNLOAD"
                    </a>
                </div>
            </div>

            <div class="result-main">
                <div class="result-slider-box shadow-2xl">
                    <ComparisonSlider 
                        before_url=data.before_url.unwrap_or_else(|| "/assets/hero_before.png".to_string()) 
                        after_url=data.image_url.unwrap_or_default()
                        before_label="BEFORE"
                        after_label="AFTER"
                    />
                </div>

                <div class="result-sidebar">
                    <div class="card settings-card">
                        <div class="params-body">
                            <div class="card-tag">
                                <Settings size={10} />
                                <span>"ASSET TELEMETRY"</span>
                            </div>
                            <div class="settings-list">
                                <div class="s-item">
                                    <span class="s-label">"IDENTITY"</span>
                                    <span class="s-value font-mono">{job_id}</span>
                                </div>
                                <div class="s-item">
                                    <span class="s-label">"STATUS"</span>
                                    <span class="s-value success">"VERIFIED"</span>
                                </div>
                                <div class="s-item">
                                    <span class="s-label">"VERSION"</span>
                                    <span class="s-value">"V2.0 STABLE"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".result-container { padding-bottom: var(--s-20); }
                .page-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); }
                .header-actions { display: flex; gap: var(--s-3); }
                
                .result-main { display: grid; grid-template-columns: 1fr 340px; gap: var(--s-12); align-items: stretch; }
                .result-slider-box { border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--glass-border); min-height: 500px; background: #000; }
                
                .settings-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); height: 100%; }
                .settings-list { flex: 1; display: flex; flex-direction: column; gap: var(--s-8); margin-top: var(--s-4); }
                .s-item { display: flex; flex-direction: column; gap: 2px; }
                .s-label { font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim)); letter-spacing: 0.1em; text-transform: uppercase; opacity: 0.6; }
                .s-value { font-size: 0.8125rem; font-weight: 750; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: hsl(var(--text)); }
                .s-value.success { color: hsl(var(--accent)); }
                
                @media (max-width: 1050px) {
                    .result-main { grid-template-columns: 1fr; }
                    .page-header { flex-direction: column; align-items: flex-start; gap: var(--s-6); }
                    .header-actions { width: 100%; }
                    .header-actions .btn { flex: 1; }
                }
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
            <h2>"Upscale Error"</h2>
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

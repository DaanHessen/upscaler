use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_navigate};
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::components::icons::{Download, AlertCircle, Settings};
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
            <Suspense fallback=|| view! { 
                <div class="loading-full-page">
                    <crate::components::icons::LoadingSpinner />
                </div> 
            }>
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
                    ("RECONSTRUCTING", "Studio Engine Engaged...".to_string(), "Gemini Vision is enhancing your asset...")
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
            <div class="processing-studio-card stagger-1">
                <div class="params-body">
                    <div class="card-tag">
                        <Settings size={10} />
                        <span>"STUDIO PULSE"</span>
                    </div>

                    <div class="studio-processing-view">
                        <div class="scanner-visual">
                            <div class="scanner-frame">
                                <div class="scanner-line"></div>
                                <div class="scanner-glow"></div>
                                <div class="scanner-lens"></div>
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
                                <span class="t-value">"STUDIO V2"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <style>
                ".polling-container { display: flex; align-items: center; justify-content: center; padding: var(--s-20) 0; min-height: 60vh; }
                .processing-studio-card { width: 100%; max-width: 500px; background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); padding: var(--s-8); box-shadow: var(--shadow-xl); }
                
                .studio-processing-view { display: flex; flex-direction: column; align-items: center; gap: var(--s-10); padding: var(--s-10) 0; }
                
                .scanner-visual { position: relative; width: 100%; display: flex; justify-content: center; }
                .scanner-frame { 
                    width: 140px; 
                    height: 140px; 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-md); 
                    background: #000; 
                    position: relative; 
                    display: flex; 
                    align-items: center; 
                    justify-content: center; 
                    overflow: hidden;
                    box-shadow: 0 0 40px hsl(var(--accent) / 0.15);
                }
                .scanner-lens {
                    width: 60px;
                    height: 60px;
                    border-radius: 50%;
                    border: 1px solid hsl(var(--accent) / 0.3);
                    background: radial-gradient(circle at center, hsl(var(--accent) / 0.1), transparent);
                    animation: pulse 2s infinite;
                }
                
                .scanner-line { 
                    position: absolute; 
                    top: 0; 
                    left: 0; 
                    width: 100%; 
                    height: 2px; 
                    background: hsl(var(--accent)); 
                    box-shadow: 0 0 15px hsl(var(--accent)); 
                    animation: scan-move 3s ease-in-out infinite; 
                    z-index: 5;
                }
                .scanner-glow {
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 40px;
                    background: linear-gradient(to bottom, hsl(var(--accent) / 0.2), transparent);
                    animation: scan-glow-move 3s ease-in-out infinite;
                    z-index: 4;
                }

                @keyframes scan-move {
                    0%, 100% { top: 0; }
                    50% { top: 100%; }
                }
                @keyframes scan-glow-move {
                    0%, 100% { top: 0; transform: scaleY(1); }
                    50% { top: calc(100% - 40px); transform: scaleY(-1); }
                }

                .processing-meta { text-align: center; display: flex; flex-direction: column; gap: var(--s-2); }
                .stage-tag { font-size: 0.625rem; font-weight: 950; color: hsl(var(--accent)); letter-spacing: 0.25em; text-transform: uppercase; }
                .stage-title { font-size: 1.5rem; font-weight: 850; letter-spacing: -0.04em; color: hsl(var(--text)); }
                .stage-desc { font-size: 0.875rem; max-width: 320px; margin: 0 auto; color: hsl(var(--text-dim)); opacity: 0.8; }

                .telemetry-bar { display: flex; gap: var(--s-10); margin-top: var(--s-4); padding-top: var(--s-8); border-top: 1px solid var(--glass-border); width: 100%; justify-content: center; }
                .telemetry-segment { display: flex; flex-direction: column; gap: 4px; align-items: center; }
                .t-label { font-size: 0.55rem; font-weight: 900; color: hsl(var(--text-dim)); opacity: 0.4; letter-spacing: 0.15em; }
                .t-value { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; color: hsl(var(--text)); }
                "
            </style>
        </div>
    }
}

#[component]
fn ResultView(data: crate::api::PollResponse, job_id: String) -> impl IntoView {
    let navigate = use_navigate();
    let (show_debug, set_show_debug) = signal(false);
    
    view! {
        <div class="result-container fade-in">
            <div class="view-result-header">
                <div class="header-main">
                    <h1 class="text-gradient stagger-1">"Upscale Complete"</h1>
                    <p class="muted stagger-2">"Your asset has been successfully reconstructed."</p>
                </div>
                <div class="header-actions stagger-3">
                    <button class="btn btn-secondary" on:click=move |_| navigate("/editor", Default::default())>"NEW UPSCALE"</button>
                    <a href=data.image_url.clone().unwrap_or_default() target="_blank" class="btn btn-primary" style="text-decoration: none;">
                        <Download size={14} />
                        "DOWNLOAD"
                    </a>
                </div>
            </div>

            <div class="result-main">
                <div class="result-slider-box shadow-2xl">
                    <ComparisonSlider 
                        images=vec![
                            (data.before_url.unwrap_or_else(|| data.image_url.clone().unwrap_or_default()), 
                             data.image_url.clone().unwrap_or_default())
                        ]
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
                            
                            <div class="card-divider" style="margin: var(--s-8) 0;"></div>
                            
                            <button 
                                class="btn btn-secondary btn-xs btn-block" 
                                on:click=move |_| set_show_debug.update(|v| *v = !*v)
                            >
                                {move || if show_debug.get() { "HIDE SYSTEM LOGS" } else { "SHOW SYSTEM LOGS" }}
                            </button>

                            {move || show_debug.get().then(|| {
                                let settings = data.prompt_settings.clone().unwrap_or_default();
                                let usage = data.usage_metadata.clone().unwrap_or_default();
                                let prompt_tokens = usage["prompt_token_count"].as_i64().unwrap_or(0);
                                let candidate_tokens = usage["candidates_token_count"].as_i64().unwrap_or(0);
                                
                                view! {
                                    <div class="debug-panel fade-in">
                                        <div class="debug-section">
                                            <span class="d-hdr">"PROMPT BUILDER"</span>
                                            <div class="d-row"><span>"Lighting"</span><span>{settings.lighting}</span></div>
                                            <div class="d-row"><span>"Focus Lock"</span><span>{settings.keep_depth_of_field.to_string()}</span></div>
                                        </div>
                                        <div class="debug-section">
                                            <span class="d-hdr">"LLM USAGE (VERTEX)"</span>
                                            <div class="d-row"><span>"Prompt Tokens"</span><span>{prompt_tokens}</span></div>
                                            <div class="d-row"><span>"Response Tokens"</span><span>{candidate_tokens}</span></div>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                </div>
            </div>

            <style>
                ".view-result-page { max-width: 1300px; margin: 0 auto; padding: 0 var(--s-4); }                .view-result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--s-16); border-bottom: 1px solid var(--glass-border); padding-bottom: var(--s-8); }
                .header-actions { display: flex; gap: var(--s-4); }
                
                .result-main { display: grid; grid-template-columns: 1fr 340px; gap: var(--s-12); align-items: stretch; }
                .result-slider-box { border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--glass-border); min-height: 500px; background: #000; display: flex; align-items: stretch; }
                
                .settings-card { background: hsl(var(--surface)); border: 1px solid var(--glass-border); border-radius: var(--radius-lg); height: 100%; }
                .settings-list { flex: 1; display: flex; flex-direction: column; gap: var(--s-4); margin-top: var(--s-4); }
                .s-item { display: flex; flex-direction: column; gap: 4px; padding: var(--s-3) 0; border-bottom: 1px solid var(--glass-border); }
                .s-item:last-child { border-bottom: none; }
                .s-label { font-size: 0.5rem; font-weight: 850; color: hsl(var(--text-dim)); letter-spacing: 0.15em; text-transform: uppercase; opacity: 0.5; }
                .s-value { font-size: 0.8125rem; font-weight: 700; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: hsl(var(--text)); }
                .s-value.success { color: hsl(var(--accent)); text-shadow: 0 0 10px hsl(var(--accent) / 0.3); }
                
                @media (max-width: 1050px) {
                    .result-main { grid-template-columns: 1fr; }
                    .page-header { flex-direction: column; align-items: flex-start; gap: var(--s-6); }
                    .header-actions { width: 100%; }
                    .header-actions .btn { flex: 1; }
                }

                .card-divider { height: 1px; background: var(--glass-border); opacity: 0.5; }
                .btn-xs { font-size: 0.6rem; padding: var(--s-3); }

                /* Debug Panel */
                .debug-panel { margin-top: var(--s-6); display: flex; flex-direction: column; gap: var(--s-6); background: rgba(0,0,0,0.3); padding: var(--s-4); border-radius: var(--radius-sm); border: 1px solid var(--glass-border); }
                .debug-section { display: flex; flex-direction: column; gap: 4px; }
                .d-hdr { font-size: 0.5rem; font-weight: 900; color: hsl(var(--accent)); letter-spacing: 0.1em; margin-bottom: 2px; }
                .d-row { display: flex; justify-content: space-between; font-size: 0.625rem; font-family: var(--font-mono); }
                .d-row span:first-child { color: hsl(var(--text-dim)); }
                .d-row span:last-child { color: hsl(var(--text)); font-weight: 600; }
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

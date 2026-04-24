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
                    ("QUEUEING", format!("Position #{} in queue", pos), "Checking on safety filters...".to_string())
                },
                "PROCESSING" => {
                    ("UPSCALING", "Upscaling Image...".to_string(), "Rescaling image...".to_string())
                },
                _ => ("FINALIZING", "Resizing & Saving...".to_string(), "Finalizing and storing your high-res asset...".to_string())
            }
        } else {
            ("STARTING", "Preparing...".to_string(), "Establishing connection and validating image...".to_string())
        }
    };

    let gs = crate::use_global_state();
    let (seconds_elapsed, set_seconds_elapsed) = signal(0);
    
    Effect::new(move |_| {
        let interval = gloo_timers::callback::Interval::new(1000, move || {
            set_seconds_elapsed.update(|s| *s += 1);
        });
        move || drop(interval)
    });

    let est_remaining = move || {
        let elapsed = seconds_elapsed.get();
        let avg = gs.avg_latency_secs.get();
        if elapsed < avg {
            format!("~{}s remaining", avg - elapsed)
        } else {
            "Finalizing...".to_string()
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
                            <p class="est-timer" style="font-size: 0.75rem; font-family: var(--font-mono); color: hsl(var(--accent)); margin-top: 8px;">
                                {move || est_remaining()}
                            </p>
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
            
            
        </div>
    }
}

#[component]
fn ResultView(data: crate::api::PollResponse, job_id: String) -> impl IntoView {
    let navigate = use_navigate();
    
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
                        <div class="params-body" style="padding: var(--s-8);">
                            <div class="card-tag" style="margin-bottom: var(--s-8);">
                                <Settings size={10} />
                                <span>"ASSET TELEMETRY"</span>
                            </div>
                            
                            <div class="pack-list" style="display: flex; flex-direction: column; gap: var(--s-3);">
                                <div class="pack-item">
                                    <div class="pack-info">
                                        <span class="pack-name">"Identity"</span>
                                        <span class="pack-credits">"Job Identifier"</span>
                                    </div>
                                    <span class="pack-price" style="font-size: 0.75rem; font-family: var(--font-mono);">{job_id.chars().take(8).collect::<String>()}</span>
                                </div>

                                <div class="pack-item">
                                    <div class="pack-info">
                                        <span class="pack-name">"Status"</span>
                                        <span class="pack-credits">"Verification"</span>
                                    </div>
                                    <span class="pack-price" style="font-size: 0.75rem; color: hsl(var(--success));">"VERIFIED"</span>
                                </div>

                                <div class="pack-item">
                                    <div class="pack-info">
                                        <span class="pack-name">"Engine"</span>
                                        <span class="pack-credits">"Version"</span>
                                    </div>
                                    <span class="pack-price" style="font-size: 0.75rem;">"V2.0 STABLE"</span>
                                </div>

                                {move || {
                                    let settings = data.prompt_settings.clone().unwrap_or_default();
                                    view! {
                                        <>
                                            <div class="pack-item">
                                                <div class="pack-info">
                                                    <span class="pack-name">"Style"</span>
                                                    <span class="pack-credits">"Engine Mode"</span>
                                                </div>
                                                <span class="pack-price" style="font-size: 0.75rem;">{data.style.clone().unwrap_or_default()}</span>
                                            </div>
                                            <div class="pack-item">
                                                <div class="pack-info">
                                                    <span class="pack-name">"Lighting"</span>
                                                    <span class="pack-credits">"Atmosphere"</span>
                                                </div>
                                                <span class="pack-price" style="font-size: 0.75rem;">{settings.lighting}</span>
                                            </div>
                                        </>
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            
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
            
        </div>
    }
}

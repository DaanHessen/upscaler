use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::components::icons::{ImageIcon, RefreshCw, AlertCircle};
use crate::auth::use_auth;
use crate::api::ApiClient;
use crate::{use_global_state};

#[component]
pub fn UploadZone() -> impl IntoView {
    let auth = use_auth();
    let global_state = use_global_state();
    let navigate = use_navigate();
    
    let (is_over, set_is_over) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Reusable file handling function
    let on_file = move |file: web_sys::File| {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.session.get().map(|s| s.access_token);
        let g_state = global_state;
        let f_clone = file.clone();
        let nav = navigate.clone();
        
        leptos::task::spawn_local(async move {
            match ApiClient::moderate(&f_clone, token.as_deref()).await {
                Ok(res) => {
                    if res.nsfw {
                        set_error.set(Some("Content violates safety guidelines (NSFW).".to_string()));
                        set_loading.set(false);
                    } else {
                        g_state.set_temp_file.set(Some(f_clone));
                        g_state.set_temp_classification.set(Some(res.detected_style));
                        nav("/configure", Default::default());
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Detection Error: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    // Use StoredValue to allow the closure to be called from multiple FnMut event handlers
    let on_file_stored = StoredValue::new(on_file);

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_is_over.set(false);
        if let Some(dt) = ev.data_transfer() {
            if let Some(files) = dt.files() {
                if let Some(f) = files.get(0) {
                    on_file_stored.with_value(|func| func(f));
                }
            }
        }
    };

    let on_input = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(f) = files.get(0) {
                on_file_stored.with_value(|func| func(f));
            }
        }
    };

    view! {
        <div class="upload-zone-wrapper">
            {move || {
                let on_drop = on_drop.clone();
                let on_input = on_input.clone();
                if loading.get() {
                    view! {
                        <div class="upload-loading">
                            <div class="scan-line"></div>
                            <RefreshCw size={32} />
                            <span class="loading-text">"SCANNING IMAGE..."</span>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div 
                            class=move || if is_over.get() { "upload-dropzone drag-over" } else { "upload-dropzone" }
                            on:dragover=move |ev| { ev.prevent_default(); set_is_over.set(true); }
                            on:dragleave=move |_| set_is_over.set(false)
                            on:drop=on_drop
                        >
                            <input type="file" id="file-upload" on:change=on_input style="display: none;" accept="image/*" />
                            
                            <label for="file-upload" class="dropzone-content">
                                <div class="icon-circle">
                                    {move || if let Some(_) = error.get() {
                                        view! { <AlertCircle size={24} custom_style="color: var(--error);".to_string() /> }.into_any()
                                    } else {
                                        view! { <ImageIcon size={24} /> }.into_any()
                                    }}
                                </div>
                                <div class="text-content">
                                    {move || match error.get() {
                                        Some(err) => view! {
                                            <h3 class="error">{err}</h3>
                                            <p>"Try another image"</p>
                                        }.into_any(),
                                        None => view! {
                                            <h3>"Select source image"</h3>
                                            <p>"or drag and drop into this area"</p>
                                        }.into_any()
                                    }}
                                </div>
                            </label>
                        </div>
                    }.into_any()
                }
            }}

            <div class="upload-footer">
                <div class="limit-box">
                    <span class="limit-label">"MAX SIZE"</span>
                    <span class="limit-value">"25MB"</span>
                </div>
                <div class="limit-box">
                    <span class="limit-label">"SYSTEM"</span>
                    <span class="limit-value">"V1.0 ALPHA"</span>
                </div>
            </div>

            <style>
                ".upload-zone-wrapper { height: 100%; display: flex; flex-direction: column; }
                .upload-dropzone { 
                    flex: 1; 
                    display: flex; 
                    flex-direction: column; 
                    align-items: center; 
                    justify-content: center; 
                    position: relative; 
                    cursor: pointer; 
                    transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); 
                    border: 1px solid var(--glass-border); 
                    border-radius: var(--radius-lg);
                    background: hsl(var(--surface));
                    box-shadow: inset 0 0 20px rgba(0,0,0,0.2);
                    overflow: hidden;
                    width: 100%;
                }
                
                .upload-dropzone::after {
                    content: '';
                    position: absolute;
                    inset: 0;
                    background: radial-gradient(circle at center, hsl(var(--accent) / 0.05), transparent 70%);
                    opacity: 0;
                    transition: opacity 0.4s;
                    pointer-events: none;
                }

                .upload-dropzone:hover { 
                    border-color: hsl(var(--accent) / 0.4);
                    background: hsl(var(--surface-raised) / 0.5); 
                    transform: translateY(-2px);
                    box-shadow: 0 20px 40px -10px rgba(0,0,0,0.5);
                }
                .upload-dropzone:hover::after { opacity: 1; }
                
                .upload-dropzone.drag-over { 
                    background: hsl(var(--accent) / 0.05); 
                    border-color: hsl(var(--accent));
                    box-shadow: 0 0 0 4px hsl(var(--accent) / 0.1);
                    transform: scale(0.99);
                }
                
                .dropzone-content { text-align: center; display: flex; flex-direction: column; align-items: center; gap: var(--s-8); width: 100%; height: 100%; justify-content: center; padding: var(--s-12); cursor: pointer; z-index: 2; }
                .icon-circle { 
                    width: 64px; 
                    height: 64px; 
                    border-radius: 50%; 
                    background: hsl(var(--surface-raised)); 
                    display: flex; 
                    align-items: center; 
                    justify-content: center; 
                    color: hsl(var(--text-muted)); 
                    border: 1px solid var(--glass-border); 
                    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                    box-shadow: 0 8px 16px rgba(0,0,0,0.3);
                }
                .upload-dropzone:hover .icon-circle {
                    background: hsl(var(--accent));
                    color: hsl(var(--bg));
                    transform: scale(1.1) rotate(5deg);
                    box-shadow: 0 0 20px hsl(var(--accent) / 0.4);
                }
                
                .text-content h3 { font-family: var(--font-heading); font-size: 1rem; font-weight: 800; margin-bottom: var(--s-1); text-transform: uppercase; letter-spacing: 0.05em; color: hsl(var(--text)); }
                .text-content p { font-size: 0.8125rem; color: hsl(var(--text-dim)); font-weight: 600; opacity: 0.8; }
                .text-content h3.error { color: hsl(var(--error)); }

                .upload-loading { 
                    flex: 1; 
                    display: flex; 
                    flex-direction: column; 
                    align-items: center; 
                    justify-content: center; 
                    gap: var(--s-6); 
                    position: relative; 
                    overflow: hidden; 
                    background: hsl(var(--surface)); 
                    border-radius: var(--radius-lg); 
                    border: 1px solid hsl(var(--accent) / 0.3); 
                    box-shadow: 0 0 40px hsl(var(--accent) / 0.1);
                }
                .loading-text { font-size: 0.75rem; font-weight: 800; color: hsl(var(--accent)); letter-spacing: 0.3rem; font-family: var(--font-mono); text-shadow: 0 0 10px hsl(var(--accent) / 0.5); }
                
                .scan-line { 
                    position: absolute; 
                    width: 100%; 
                    height: 100px; 
                    background: linear-gradient(to bottom, transparent, hsl(var(--accent) / 0.2), transparent); 
                    top: -100px; 
                    animation: scan 2.5s ease-in-out infinite; 
                    border-bottom: 1px solid hsl(var(--accent) / 0.5);
                    box-shadow: 0 20px 40px -10px hsl(var(--accent) / 0.2); 
                }
                @keyframes scan { 
                    0% { top: -100px; opacity: 0; } 
                    20% { opacity: 1; }
                    80% { opacity: 1; }
                    100% { top: 100%; opacity: 0; } 
                }

                .upload-footer { margin-top: var(--s-6); display: flex; justify-content: space-between; border-top: 1px solid var(--glass-border); padding-top: var(--s-6); }
                .limit-box { display: flex; flex-direction: column; gap: var(--s-1); }
                .limit-label { font-size: 0.625rem; font-weight: 900; color: hsl(var(--text-dim)); letter-spacing: 0.15em; text-transform: uppercase; }
                .limit-value { font-size: 0.75rem; font-weight: 700; color: hsl(var(--text)); font-family: var(--font-mono); opacity: 0.9; }
                "
            </style>
        </div>
    }
}

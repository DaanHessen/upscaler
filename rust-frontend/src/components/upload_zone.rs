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
                            <span class="loading-text">"ANALYZING SIGNAL..."</span>
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
                                        view! { <AlertCircle size={24} style="color: var(--error);".to_string() /> }.into_any()
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
                    <span class="limit-label">"MAX PAYLOAD"</span>
                    <span class="limit-value">"25MB"</span>
                </div>
                <div class="limit-box">
                    <span class="limit-label">"ARCHITECTURE"</span>
                    <span class="limit-value">"V7.1 STABLE"</span>
                </div>
            </div>

            <style>
                ".upload-zone-wrapper { height: 100%; display: flex; flex-direction: column; }
                .upload-dropzone { flex: 1; min-height: 250px; display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; cursor: pointer; transition: all 0.2s; border: 1px dashed var(--border-color); border-radius: 8px; }
                .upload-dropzone:hover { border-color: var(--accent); background: rgba(88, 166, 255, 0.02); }
                .upload-dropzone.drag-over { background: rgba(88, 166, 255, 0.05); border-color: var(--accent); }
                
                .dropzone-content { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1rem; width: 100%; height: 100%; justify-content: center; padding: 2rem; cursor: pointer; }
                .icon-circle { width: 48px; height: 48px; border-radius: 50%; background: var(--surface-color); display: flex; align-items: center; justify-content: center; color: var(--text-muted); border: 1px solid var(--border-color); }
                
                .text-content h3 { font-size: 0.9rem; font-weight: 700; margin-bottom: 0.25rem; }
                .text-content p { font-size: 0.75rem; color: var(--text-muted); font-weight: 600; }
                .text-content h3.error { color: var(--error); }

                .upload-loading { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1.5rem; position: relative; overflow: hidden; background: rgba(88, 166, 255, 0.02); border-radius: 8px; border: 1px solid var(--border-color); }
                .loading-text { font-size: 0.75rem; font-weight: 800; color: var(--accent); letter-spacing: 0.1em; font-family: var(--font-mono); }
                
                .scan-line { position: absolute; width: 100%; height: 2px; background: linear-gradient(90deg, transparent, var(--accent), transparent); top: 0; animation: scan 2s linear infinite; }
                @keyframes scan { from { top: 0; } to { top: 100%; } }

                .upload-footer { margin-top: 1.5rem; display: flex; justify-content: space-between; border-top: 1px solid var(--border-color); padding-top: 1.5rem; }
                .limit-box { display: flex; flex-direction: column; gap: 0.25rem; }
                .limit-label { font-size: 0.6rem; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
                .limit-value { font-size: 0.75rem; font-weight: 700; color: var(--text-color); font-family: var(--font-mono); }
                "
            </style>
        </div>
    }
}

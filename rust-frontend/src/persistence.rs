use leptos::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

const DB_NAME: &str = "upsyl_studio_db";
const STORE_NAME: &str = "temp_files";
const FILE_KEY: &str = "uploaded_image";

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSettings {
    pub quality: String,
    pub style: String,
    pub temperature: f32,
    pub keep_depth_of_field: bool,
    pub lighting: String,
    pub thinking_level: String,
    pub seed: Option<u32>,
    pub theme: String,
    #[serde(default = "default_tool")]
    pub active_tool: String,
    #[serde(default = "default_medium")]
    pub target_medium: String,
    #[serde(default = "default_render")]
    pub render_style: String,
    #[serde(default = "default_ratio")]
    pub target_aspect_ratio: String,
}

fn default_tool() -> String { "UPSCALE".to_string() }
fn default_medium() -> String { "3D Render".to_string() }
fn default_render() -> String { "Photorealistic".to_string() }
fn default_ratio() -> String { "16:9".to_string() }

pub fn save_classification(style: Option<String>) {
    if let Some(s) = style {
        let _ = LocalStorage::set("upsyl_temp_classification", s);
    } else {
        LocalStorage::delete("upsyl_temp_classification");
    }
}

pub fn load_classification() -> Option<String> {
    LocalStorage::get("upsyl_temp_classification").ok()
}

pub fn save_settings(settings: &UserSettings) {
    let _ = LocalStorage::set("upsyl_user_settings", settings);
}

pub fn load_settings() -> Option<UserSettings> {
    LocalStorage::get("upsyl_user_settings").ok()
}

pub async fn save_file(file: &web_sys::File) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let db_request = window.indexed_db()?.ok_or("No IDB")?.open_with_u32(DB_NAME, 1)?;

    let on_upgrade = Closure::once(move |ev: web_sys::IdbVersionChangeEvent| {
        let target = ev.target().expect("Event should have target");
        let request = target.unchecked_into::<web_sys::IdbOpenDbRequest>();
        let db: web_sys::IdbDatabase = request.result().unwrap().unchecked_into();
        let _ = db.create_object_store(STORE_NAME);
    });
    db_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();

    let res = wait_for_request(db_request.into()).await?;
    let db: web_sys::IdbDatabase = res.unchecked_into();
    
    let transaction: web_sys::IdbTransaction = db.transaction_with_str_and_mode(STORE_NAME, web_sys::IdbTransactionMode::Readwrite)?;
    let store: web_sys::IdbObjectStore = transaction.object_store(STORE_NAME)?;
    store.put_with_key(file, &JsValue::from_str(FILE_KEY))?;
    
    Ok(())
}

pub async fn load_file() -> Option<web_sys::File> {
    let window = web_sys::window()?;
    let idb = window.indexed_db().ok()??;
    
    let open_request = idb.open_with_u32(DB_NAME, 1).ok()?;
    
    let on_upgrade = Closure::once(move |ev: web_sys::IdbVersionChangeEvent| {
        let target = ev.target().expect("Event should have target");
        let request = target.unchecked_into::<web_sys::IdbOpenDbRequest>();
        let db: web_sys::IdbDatabase = request.result().unwrap().unchecked_into();
        let _ = db.create_object_store(STORE_NAME);
    });
    open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();

    let res = wait_for_request(open_request.into()).await.ok()?;
    let db: web_sys::IdbDatabase = res.unchecked_into();
    
    let transaction: web_sys::IdbTransaction = db.transaction_with_str(STORE_NAME).ok()?;
    let store: web_sys::IdbObjectStore = transaction.object_store(STORE_NAME).ok()?;
    let request: web_sys::IdbRequest = store.get(&JsValue::from_str(FILE_KEY)).ok()?;
    
    let res = wait_for_request(request).await.ok()?;
    res.dyn_into::<web_sys::File>().ok()
}

#[allow(dead_code)]
pub async fn clear_persistence() {
    let _ = LocalStorage::delete("upsyl_temp_classification");
    if let Some(window) = web_sys::window() {
        if let Ok(Some(idb)) = window.indexed_db() {
            if let Ok(req) = idb.open_with_u32(DB_NAME, 1) {
                if let Ok(res) = wait_for_request(req.into()).await {
                    if let Ok(db) = res.dyn_into::<web_sys::IdbDatabase>() {
                        if let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, web_sys::IdbTransactionMode::Readwrite) {
                            if let Ok(store) = tx.object_store(STORE_NAME) {
                                let _request: Result<web_sys::IdbRequest, JsValue> = store.clear();
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn wait_for_request(request: web_sys::IdbRequest) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_success = Closure::once(move |ev: web_sys::Event| {
            let request: web_sys::IdbRequest = ev.target().unwrap().unchecked_into();
            resolve.call1(&JsValue::undefined(), &request.result().unwrap_or(JsValue::NULL)).unwrap();
        });
        let on_error = Closure::once(move |ev: web_sys::Event| {
            reject.call1(&JsValue::undefined(), &ev).unwrap();
        });
        
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        
        on_success.forget();
        on_error.forget();
    });
    
    JsFuture::from(promise).await
}

use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitResponse {
    pub success: bool,
    pub job_id: Uuid,
    pub final_style: String,
    pub tool_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PollResponse {
    pub status: String,
    pub image_url: Option<String>,
    pub preview_url: Option<String>,
    pub before_url: Option<String>,
    pub error: Option<String>,
    pub queue_position: Option<i64>,
    pub prompt_settings: Option<PromptSettings>,
    pub usage_metadata: Option<serde_json::Value>,
    pub latency_ms: Option<i32>,
    pub style: Option<String>,
    pub tool_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryItem {
    pub id: Uuid,
    pub status: String,
    pub created_at: String,
    pub quality: String,
    pub style: Option<String>,
    pub temperature: f32,
    pub tool_type: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub preview_url: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub prompt_settings: serde_json::Value,
    pub usage_metadata: serde_json::Value,
    pub latency_ms: i32,
    pub credits_charged: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModerateResponse {
    pub nsfw: bool,
    pub detected_style: String,
    pub preview_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    #[serde(default)]
    pub keep_depth_of_field: bool,
    #[serde(default)]
    pub lighting: String,
    #[serde(default)]
    pub thinking_level: String,
    #[serde(default)]
    pub seed: Option<u32>,
    #[serde(default)]
    pub target_medium: String,
    #[serde(default)]
    pub render_style: String,
    #[serde(default)]
    pub target_aspect_ratio: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BalanceResponse {
    pub credits: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CheckoutResponse {
    pub url: String,
}

pub struct ApiClient;

impl ApiClient {
    fn authenticated_request(method: &str, endpoint: &str, token: Option<&str>) -> RequestBuilder {
        let req = match method {
            "GET" => Request::get(endpoint),
            "POST" => Request::post(endpoint),
            _ => Request::get(endpoint),
        };

        if let Some(t) = token {
            req.header("Authorization", &format!("Bearer {}", t))
        } else {
            req
        }
    }

    pub async fn get_balance(token: Option<&str>) -> Result<i32, String> {
        let resp = Self::authenticated_request("GET", "/api/balance", token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if resp.ok() {
            let data: BalanceResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data.credits)
        } else if resp.status() == 401 {
            Err("AUTH_EXPIRED".to_string())
        } else {
            Err(format!("Error fetching balance: {}", resp.status()))
        }
    }

    pub async fn get_history(token: Option<&str>) -> Result<Vec<HistoryItem>, String> {
        let resp = Self::authenticated_request("GET", "/api/history", token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let json_text = resp.text().await.map_err(|e| e.to_string())?;
            
            let data: Vec<HistoryItem> = serde_json::from_str(&json_text).map_err(|e| {
                let err_msg = format!("History Deserialization Error: {}. Raw length: {}", e, json_text.len());
                leptos::logging::error!("{}", err_msg);
                err_msg
            })?;
            
            Ok(data)
        } else if resp.status() == 401 {
            Err("AUTH_EXPIRED".to_string())
        } else {
            Err(format!("Error fetching history: {}", resp.status()))
        }
    }

    pub async fn poll_job(job_id: Uuid, token: Option<&str>) -> Result<PollResponse, String> {
        let endpoint = format!("/api/upscales/{}", job_id);
        let resp = Self::authenticated_request("GET", &endpoint, token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let data: PollResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else {
            Err(format!("Error polling job: {}", resp.status()))
        }
    }

    pub async fn moderate(
        file: &web_sys::File,
        token: Option<&str>
    ) -> Result<ModerateResponse, String> {
        let form_data = web_sys::FormData::new().map_err(|e| format!("{:?}", e))?;
        form_data.append_with_blob("image", file).map_err(|e| format!("{:?}", e))?;

        let url = "/api/moderate";
        let mut req = Request::post(url);
        
        if let Some(t) = token {
            req = req.header("Authorization", &format!("Bearer {}", t));
        }

        let resp = req.body(form_data)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let data: ModerateResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["error"].as_str().unwrap_or("Moderation failed");
            Err(msg.to_string())
        }
    }

    pub async fn submit_upscale(
        file: &web_sys::File,
        quality: &str,
        style: &str,
        temperature: f32,
        prompt_settings: &PromptSettings,
        tool_type: &str,
        token: Option<&str>
    ) -> Result<SubmitResponse, String> {
        let form_data = web_sys::FormData::new().map_err(|e| format!("{:?}", e))?;
        form_data.append_with_blob("image", file).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("quality", quality).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("style", style).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("temperature", &temperature.to_string()).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("tool_type", tool_type).map_err(|e| format!("{:?}", e))?;
        
        let settings_json = serde_json::to_string(prompt_settings).unwrap_or_default();
        form_data.append_with_str("prompt_settings", &settings_json).map_err(|e| format!("{:?}", e))?;

        let url = "/api/upscale";
        let mut req = Request::post(url);
        
        if let Some(t) = token {
            req = req.header("Authorization", &format!("Bearer {}", t));
        }

        let resp = req.body(form_data)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        let body_text = resp.text().await.map_err(|e| e.to_string())?;

        if status >= 200 && status < 300 {
            let data: SubmitResponse = serde_json::from_str(&body_text)
                .map_err(|_| format!("Invalid response format: {}", body_text))?;
            Ok(data)
        } else {
            let err_msg = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str().map(|s| s.to_string())))
                .unwrap_or_else(|| {
                    if body_text.is_empty() {
                        format!("Server returned status {}", status)
                    } else {
                        // Truncate if it's too long (e.g. HTML error page)
                        if body_text.len() > 100 {
                            format!("Error {}: {}...", status, &body_text[..100])
                        } else {
                            format!("Error {}: {}", status, body_text)
                        }
                    }
                });
            Err(err_msg)
        }
    }

    pub async fn get_health() -> Result<bool, String> {
        let resp = Request::get("/api/health")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(resp.ok())
    }

    pub async fn change_password(token: Option<&str>, new_password: &str) -> Result<(), String> {
        let body = serde_json::json!({ "new_password": new_password });
        let resp = Self::authenticated_request("POST", "/api/auth/change-password", token)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            Ok(())
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["error"].as_str().unwrap_or("Failed to change password");
            Err(msg.to_string())
        }
    }

    pub async fn create_checkout_session(
        token: Option<&str>,
        tier: &str,
        success_url: &str,
        cancel_url: &str
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "tier": tier,
            "success_url": success_url,
            "cancel_url": cancel_url
        });

        let resp = Self::authenticated_request("POST", "/api/checkout", token)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let data: CheckoutResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data.url)
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["error"].as_str().unwrap_or("Checkout failed");
            Err(msg.to_string())
        }
    }
}

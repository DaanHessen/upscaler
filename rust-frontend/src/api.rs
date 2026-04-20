use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitResponse {
    pub success: bool,
    pub job_id: Uuid,
    pub final_style: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PollResponse {
    pub status: String,
    pub image_url: Option<String>,
    pub before_url: Option<String>,
    pub error: Option<String>,
    pub queue_position: Option<i64>,
    pub prompt_settings: Option<PromptSettings>,
    pub usage_metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryItem {
    pub id: Uuid,
    pub status: String,
    pub created_at: String,
    pub quality: String,
    pub style: Option<String>,
    pub temperature: f32,
    pub image_url: Option<String>,
    pub error: Option<String>,
    pub prompt_settings: serde_json::Value,
    pub usage_metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModerateResponse {
    pub nsfw: bool,
    pub detected_style: String,
    pub preview_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptSettings {
    pub keep_aspect_ratio: bool,
    pub keep_depth_of_field: bool,
    pub lighting: String,
    pub thinking_level: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BalanceResponse {
    pub credits: i32,
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
        let resp = Self::authenticated_request("GET", "/balance", token)
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
        let resp = Self::authenticated_request("GET", "/history", token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            let data: Vec<HistoryItem> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else if resp.status() == 401 {
            Err("AUTH_EXPIRED".to_string())
        } else {
            Err(format!("Error fetching history: {}", resp.status()))
        }
    }

    pub async fn poll_job(job_id: Uuid, token: Option<&str>) -> Result<PollResponse, String> {
        let endpoint = format!("/upscales/{}", job_id);
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

        let url = "/moderate";
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
        token: Option<&str>
    ) -> Result<SubmitResponse, String> {
        let form_data = web_sys::FormData::new().map_err(|e| format!("{:?}", e))?;
        form_data.append_with_blob("image", file).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("quality", quality).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("style", style).map_err(|e| format!("{:?}", e))?;
        form_data.append_with_str("temperature", &temperature.to_string()).map_err(|e| format!("{:?}", e))?;
        
        let settings_json = serde_json::to_string(prompt_settings).unwrap_or_default();
        form_data.append_with_str("prompt_settings", &settings_json).map_err(|e| format!("{:?}", e))?;

        let url = "/upscale";
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
            let data: SubmitResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else {
            let err_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let msg = err_body["error"].as_str().unwrap_or("Submission failed");
            Err(msg.to_string())
        }
    }

    pub async fn get_health() -> Result<bool, String> {
        let resp = Request::get("/health")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(resp.ok())
    }
}

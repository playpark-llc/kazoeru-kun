use gloo_net::http::Request;
use serde_json::Value;
use crate::env;
use chrono::{Local, Duration};
// use gloo::console::{log, error, warn, info};

// エラー型の定義
#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    ParseError(String),
    RequestFailed,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::RequestFailed => write!(f, "Request failed"),
        }
    }
}

impl std::error::Error for ApiError {}

// APIクライアント構造体
pub struct ApiClient;

impl ApiClient {
    // ステータス履歴を取得
    pub async fn get_status_history() -> Result<String, ApiError> {
        let response = Request::get(&format!("{}/{}", env::APP_API_ROOT, "status_history"))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::RequestFailed);
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let status = json
            .get("result")
            .and_then(|r| r.get("current_status"))
            .and_then(|cs| cs.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("CLEAN");

        Ok(status.to_string())
    }

    // セッションを送信（sessionクッキーのみ）
    pub async fn send_session() -> Result<(), ApiError> {

        let request = Request::post(&format!("{}/{}", env::APP_API_ROOT, "session"))
            .credentials(web_sys::RequestCredentials::Include);

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if response.ok() {
            Ok(())
        } else {
            Err(ApiError::RequestFailed)
        }
    }

    // セッションを送信（sessionクッキーのみ）
    pub async fn get_count_history() -> Result<u32, ApiError> {
        // 現在の日付を取得
        let now = Local::now();
        let end_date = now - Duration::days(1);
        // 1週間前の日付を計算
        let start_date = end_date - Duration::days(8);
        
        // YYYY-MM-DD形式でフォーマット
        let start_date_str = start_date.format("%Y-%m-%d").to_string();
        let end_date_str = end_date.format("%Y-%m-%d").to_string();
        let response = Request::get(
                &format!(
                    "{}/{}?start_date={}&end_date={}",
                    env::APP_API_ROOT,
                    "session_counter",
                    start_date_str,
                    end_date_str,
                )
            )
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.ok() {
            return Err(ApiError::RequestFailed);
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let count = json
            .get("result")
            .and_then(|r| r.get("sum_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        Ok(count)
    }
}

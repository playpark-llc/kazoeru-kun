use rocket::{
    http::{Cookie, CookieJar, SameSite},
    time::OffsetDateTime,
};

/// セッションCookieを設定するユーティリティ関数
/// 
/// # Arguments
/// 
/// * `cookies` - RocketのCookieJar
/// * `session_id` - セッションID
/// * `expiration` - Cookie有効期限
/// 
/// # Example
/// 
/// ```
/// use chrono::Local;
/// use crate::http::cookie_utils::set_session_cookie;
/// 
/// let expiration = Local::now() + chrono::Duration::hours(24);
/// set_session_cookie(&cookies, "session-uuid", expiration);
/// ```
pub fn set_session_cookie(
    cookies: &CookieJar<'_>,
    session_id: &str,
    expiration: chrono::DateTime<chrono::Local>,
) {
    // chronoのDateTimeをtime::OffsetDateTimeに変換
    let offset_expiration = OffsetDateTime::from_unix_timestamp(expiration.timestamp())
        .expect("Failed to convert timestamp to OffsetDateTime");
    
    // Cookieを作成し、有効期限を設定
    let mut cookie = Cookie::new("session", session_id.to_string());
    cookie.set_expires(offset_expiration);
    // クロスサイトリクエスト（異なるドメイン間）でCookieを送受信するための設定
    // SameSite=None: クロスサイトでも送信可能
    // Secure: HTTPS必須（SameSite=Noneの必須要件）
    cookie.set_same_site(SameSite::None);
    cookie.set_secure(true);

    // デバッグログ出力
    log::debug!("Setting session cookie: id={}, expires={:?}", session_id, offset_expiration);

    cookies.add(cookie);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::{get, routes, State};
    use std::sync::{Arc, Mutex};

    // テスト用のハンドラー
    #[get("/test-cookie")]
    fn test_cookie_handler(cookies: &CookieJar<'_>, state: &State<Arc<Mutex<Vec<String>>>>) {
        let expiration = Local::now() + chrono::Duration::hours(24);
        let session_id = "test-session-id";
        
        set_session_cookie(cookies, session_id, expiration);
        
        // セッションIDを記録
        state.lock().unwrap().push(session_id.to_string());
    }

    #[test]
    fn test_set_session_cookie() {
        // Rocketアプリケーションを作成
        let state = Arc::new(Mutex::new(Vec::<String>::new()));
        let rocket = rocket::build()
            .manage(state.clone())
            .mount("/", routes![test_cookie_handler]);
        
        let client = Client::tracked(rocket).expect("valid rocket instance");
        
        // リクエストを送信
        let response = client.get("/test-cookie").dispatch();
        
        // ステータスコードを確認
        assert_eq!(response.status(), Status::Ok);
        
        // Cookieが設定されていることを確認
        let cookies = response.cookies();
        let session_cookie = cookies.get("session");
        
        assert!(session_cookie.is_some());
        assert_eq!(session_cookie.unwrap().value(), "test-session-id");
        
        // セッションIDが記録されていることを確認
        assert_eq!(state.lock().unwrap().len(), 1);
        assert_eq!(state.lock().unwrap()[0], "test-session-id");
    }

    #[test]
    fn test_cookie_expiration_conversion() {
        // 特定の日時を作成
        let local_time = Local.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        
        // タイムスタンプに変換
        let timestamp = local_time.timestamp();
        
        // OffsetDateTimeに変換できることを確認
        let offset_time = OffsetDateTime::from_unix_timestamp(timestamp);
        
        assert!(offset_time.is_ok());
        
        let offset_time = offset_time.unwrap();
        assert_eq!(offset_time.unix_timestamp(), timestamp);
    }

    #[test]
    fn test_cookie_with_future_expiration() {
        // 未来の日時を作成（翌日の0:00:00）
        let tomorrow = Local::now()
            .date_naive()
            .succ_opt()
            .and_then(|tomorrow| tomorrow.and_hms_opt(0, 0, 0))
            .and_then(|dt| Local.from_local_datetime(&dt).single())
            .unwrap_or_else(|| Local::now() + chrono::Duration::hours(24));
        
        // タイムスタンプが現在より未来であることを確認
        assert!(tomorrow.timestamp() > Local::now().timestamp());
        
        // OffsetDateTimeに変換できることを確認
        let offset_time = OffsetDateTime::from_unix_timestamp(tomorrow.timestamp());
        assert!(offset_time.is_ok());
    }
}
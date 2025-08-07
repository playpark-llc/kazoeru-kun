use rocket::{get, post, put, http::CookieJar, serde::json::Json, State};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::models::response::{ErrorResponse, Response, SessionCount, SessionCounterSummary};
use crate::repositories::{SessionRepository, SessionCounterRepository};
use crate::services::{SessionService, SyncService};
use crate::{DateRange, get_date_range_local};

/// POST /session - セッションのクリックカウンタをインクリメント
#[post("/session")]
pub async fn create_session(
    session_repo: &State<Arc<dyn SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Response<SessionCount>>, Json<ErrorResponse>> {
    // リクエスト処理とレスポンス返却のみ
    // ビジネスロジックはサービス層に委譲
    
    let session_id = cookies.get("session").map(|cookie| cookie.value());
    
    let session_service = SessionService::new(session_repo.inner().clone());
    
    match session_service.increment_session(session_id).await {
        Ok(session_response) => {
            // 新規セッションの場合はクッキーを設定
            if session_response.is_new {
                use crate::{create_expiration, create_session_cookie_with_expiration};
                let expiration = create_expiration();
                create_session_cookie_with_expiration(cookies, expiration);
            }
            
            Ok(Json(Response {
                message: session_response.message,
                result: SessionCount { count: session_response.count },
            }))
        }
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to increment session: {:?}", e),
        })),
    }
}

/// PUT /session - RedisからMySQLへのセッションデータ同期
#[put("/session")]
pub async fn sync_sessions(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    db: &State<DatabaseConnection>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // ビジネスロジックはサービス層に委譲
    let sync_service = SyncService::new(
        session_repo.inner().clone(),
        counter_repo.inner().clone(),
        Arc::new(db.inner().clone()),
    );
    
    match sync_service.sync_sessions_to_database().await {
        Ok(sync_result) => {
            Ok(Json(Response {
                message: sync_result.message,
                result: if sync_result.success { 
                    "Success".to_string() 
                } else { 
                    "Failed".to_string() 
                },
            }))
        }
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to sync sessions: {:?}", e),
        })),
    }
}

/// GET /session_counter - セッションカウンタのサマリを取得
#[get("/session_counter?<date_range..>")]
pub async fn get_session_counter(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(session_repo, counter_repo, date_range).await
}

/// GET /count_summary - 後方互換性のためのエンドポイント
#[get("/count_summary?<date_range..>")]
pub async fn get_count_summary(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(session_repo, counter_repo, date_range).await
}

// 共通実装
async fn get_session_counter_impl(
    _session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    let date_range = get_date_range_local(date_range);
    
    // 指定期間の合計を取得
    let sum_count = counter_repo
        .sum_count_between(date_range.start_date, date_range.end_date)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get count summary: {:?}", e),
            })
        })?;
    
    use sea_orm::prelude::Decimal;
    let res = Response {
        message: "指定期間のクリック数を取得しました".to_string(),
        result: SessionCounterSummary { 
            sum_count: Decimal::from(sum_count) 
        },
    };
    Ok(Json(res))
}
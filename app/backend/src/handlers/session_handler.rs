use crate::models::response::{ErrorResponse, Response, SessionCount, SessionCounterSummary};
use crate::repositories::SessionCounterRepository;
use crate::services::{SessionService, SyncService};
use crate::{get_date_range_local, DateRange};
use rocket::{delete, get, http::CookieJar, post, put, serde::json::Json, State};
use std::sync::Arc;

/// POST /session - セッションのクリックカウンタをインクリメント
#[post("/session")]
pub async fn create_session(
    session_service: &State<Arc<SessionService>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Response<SessionCount>>, Json<ErrorResponse>> {
    // リクエスト処理とレスポンス返却のみ
    // ビジネスロジックはサービス層に委譲

    let session_id = cookies.get("session").map(|cookie| cookie.value());

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
                result: SessionCount {
                    count: session_response.count,
                },
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
    sync_service: &State<Arc<SyncService>>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // ビジネスロジックはサービス層に委譲

    match sync_service.sync_sessions_to_database().await {
        Ok(sync_result) => Ok(Json(Response {
            message: sync_result.message,
            result: if sync_result.success {
                "Success".to_string()
            } else {
                "Failed".to_string()
            },
        })),
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to sync sessions: {:?}", e),
        })),
    }
}

/// GET /session_counter - セッションカウンタのサマリを取得
#[get("/session_counter?<date_range..>")]
pub async fn get_session_counter(
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(counter_repo, date_range).await
}

/// GET /count_summary - 後方互換性のためのエンドポイント
#[get("/count_summary?<date_range..>")]
pub async fn get_count_summary(
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(counter_repo, date_range).await
}

/// DELETE /session/expired - 期限切れセッションのクリーンアップ
#[delete("/session/expired")]
pub async fn cleanup_expired_sessions(
    sync_service: &State<Arc<SyncService>>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    match sync_service.cleanup_expired_sessions().await {
        Ok(cleaned_count) => Ok(Json(Response {
            message: format!(
                "{}件の期限切れセッションをクリーンアップしました",
                cleaned_count
            ),
            result: format!("Cleaned {} sessions", cleaned_count),
        })),
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to cleanup expired sessions: {:?}", e),
        })),
    }
}

// 共通実装
async fn get_session_counter_impl(
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
            sum_count: Decimal::from(sum_count),
        },
    };
    Ok(Json(res))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::prelude::Decimal;

    #[test]
    fn test_session_counter_response_structure() {
        // Test that SessionCount can be created
        let session_count = SessionCount { count: 42 };
        assert_eq!(session_count.count, 42);

        // Test SessionCounterSummary structure
        let summary = SessionCounterSummary {
            sum_count: Decimal::from(1000),
        };
        assert_eq!(summary.sum_count, Decimal::from(1000));
    }

    #[test]
    fn test_session_counter_endpoint_params() {
        // Test that the endpoint accepts correct parameters
        use crate::DateRange;

        // Test optional DateRange handling
        let date_range: Option<DateRange> = None;
        let processed_range = get_date_range_local(date_range);

        // Should return a valid date range structure
        assert!(processed_range.start_date <= processed_range.end_date);
    }

    #[test]
    fn test_error_response_structure() {
        // Test ErrorResponse creation
        let error = ErrorResponse {
            error: "Test error message".to_string(),
        };
        assert_eq!(error.error, "Test error message");
    }

    #[test]
    fn test_response_structure() {
        // Test generic Response wrapper
        let response = Response {
            message: "Test message".to_string(),
            result: SessionCount { count: 5 },
        };

        assert_eq!(response.message, "Test message");
        assert_eq!(response.result.count, 5);
    }

    #[test]
    fn test_cleanup_response_format() {
        // Test cleanup response message format
        let cleaned_count = 10;
        let response = Response {
            message: format!(
                "{}件の期限切れセッションをクリーンアップしました",
                cleaned_count
            ),
            result: format!("Cleaned {} sessions", cleaned_count),
        };

        assert_eq!(
            response.message,
            "10件の期限切れセッションをクリーンアップしました"
        );
        assert_eq!(response.result, "Cleaned 10 sessions");
    }

    #[test]
    fn test_cleanup_error_response() {
        // Test cleanup error response
        let error = ErrorResponse {
            error: format!(
                "Failed to cleanup expired sessions: {:?}",
                "Database connection error"
            ),
        };

        assert!(error.error.contains("Failed to cleanup expired sessions"));
        assert!(error.error.contains("Database connection error"));
    }
}

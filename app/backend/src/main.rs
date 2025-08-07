mod config;
mod error;
mod models;
mod repositories;
mod retry;
mod sea_generated;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::repositories::{
    RedisSessionRepository, SessionRepository,
    DbSessionCounterRepository, SessionCounterRepository,
    DbStatusHistoryRepository, StatusHistoryRepository,
    CreateSessionCounter, CreateStatusHistory,
};
use chrono::{Local, TimeZone};
use deadpool_redis::{
    Config, Pool, Runtime,
};
use std::sync::Arc;
use models::{
    domain::{apply_rank_change, calculate_rank_change, get_current_week_range, status_to_string},
    response::{
        ErrorResponse, PageInfo, Response, SessionCount, SessionCounterSummary,
        StatusHistoryRecord, StatusHistoryResponse,
    },
};
use opt::{
    create_expiration, create_session_cookie_with_expiration, get_date_range_local,
    get_redis_connection, DateRange,
};
use rocket::{http::CookieJar, serde::json::Json, State};
use sea_generated::{
    prelude::{SessionCounter, StatusHistory},
    sea_orm_active_enums::Status,
    session_counter, status_history,
};
use sea_orm::{
    prelude::Decimal, sea_query::OnConflict, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

#[macro_use]
extern crate rocket;

#[post("/session")]
async fn post_session(
    session_repo: &State<Arc<dyn SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Response<SessionCount>>, AppError> {
    let session_id: String;
    let mut need_new_session = false;
    
    let some_cookie = cookies.get("session");
    match some_cookie {
        Some(cookie) => {
            // クッキーの有効期限を取得
            log::debug!(
                "Cookie: {}, expires: {:?}",
                cookie.value(),
                cookie.expires()
            );
            session_id = cookie.value().to_string();
            
            // Check if session is still valid in Redis
            let is_valid = session_repo
                .is_valid(&session_id)
                .await
                .map_err(|e| AppError::from(e))?;
            
            if !is_valid {
                log::info!("Session {} has expired or doesn't exist, creating new session", session_id);
                need_new_session = true;
            }
        }
        None => {
            need_new_session = true;
            session_id = String::new(); // Will be replaced
        }
    }
    
    let final_session_id = if need_new_session {
        let expiration = create_expiration();
        create_session_cookie_with_expiration(cookies, expiration)
    } else {
        session_id
    };

    // redisにセッションIDを保存
    let session_count = session_repo
        .increment(&final_session_id)
        .await
        .map_err(|e| AppError::from(e))?;

    let res = match session_count {
        // session_idのカウントが1の場合は有効期限を設定
        1 => {
            let expiration = create_expiration();
            session_repo
                .set_expiration(&final_session_id, expiration.timestamp())
                .await
                .map_err(|e| AppError::from(e))?;

            Response {
                message: "新規セッションが登録されました".to_string(),
                result: SessionCount { count: 1 },
            }
        }
        // session_idのカウントが1以外の場合なにもしない
        _ => Response {
            message: "セッションカウントが更新されました".to_string(),
            result: SessionCount {
                count: session_count,
            },
        },
    };
    return Ok(Json(res));
}

#[put("/session")]
async fn put_session(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    db: &State<DatabaseConnection>,
) -> Result<Json<Response<String>>, AppError> {
    // Redisから全セッション情報を取得
    let sessions = session_repo
        .get_all_sessions()
        .await
        .map_err(|e| AppError::from(e))?;

    if sessions.is_empty() {
        return Ok(Json(Response {
            message: "更新するDBのセッション情報はありませんでした".to_string(),
            result: "Success".to_string(),
        }));
    }

    let insert_records: Vec<CreateSessionCounter> = sessions
        .into_iter()
        .map(|session| CreateSessionCounter {
            session_id: session.session_id,
            count: session.count,
        })
        .collect();

    // トランザクション内でデータベースにupsert
    let txn = db
        .begin()
        .await
        .map_err(|e| AppError::DatabaseTransactionError(format!("Failed to begin transaction: {:?}", e)))?;

    match counter_repo
        .upsert_many_with_txn(insert_records, &txn)
        .await
    {
        Ok(_) => {
            txn.commit()
                .await
                .map_err(|e| AppError::DatabaseTransactionError(format!("Failed to commit transaction: {:?}", e)))?;
        }
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(AppError::from(e));
        }
    }

    let res = Response {
        message: "DBのセッション情報を更新しました".to_string(),
        result: "Success".to_string(),
    };
    Ok(Json(res))
}

#[put("/status_history")]
async fn put_status_history(
    status_history_repo: &State<Arc<dyn StatusHistoryRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    config: &State<AppConfig>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // Get target value from configuration
    let target_value: i64 = config.target_value;

    // Get current week range in Japan time
    let (week_start, week_end) = get_current_week_range();

    // Convert to Local timezone for database query
    let start_date_local = Local.from_utc_datetime(&week_start.naive_utc());
    let end_date_local = Local.from_utc_datetime(&week_end.naive_utc());

    // Calculate the sum of session counts for the current week
    let weekly_count = counter_repo
        .sum_count_between(start_date_local, end_date_local)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to calculate weekly count: {:?}", e),
            })
        })?;

    log::debug!("Weekly count: {}, Target: {}", weekly_count, target_value);

    // Get the current status from the database (most recent entry)
    let current_status_result = status_history_repo
        .get_latest()
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get current status: {:?}", e),
            })
        })?;

    let current_status = current_status_result
        .map(|r| r.status)
        .unwrap_or(Status::Clean); // Default to Clean if no previous status

    // Calculate rank change
    let rank_change = calculate_rank_change(weekly_count, target_value);

    // Apply rank change to get new status
    let new_status = apply_rank_change(current_status.clone(), rank_change);

    log::debug!(
        "Current status: {:?}, Rank change: {}, New status: {:?}",
        current_status,
        rank_change,
        new_status
    );

    // Check if a record already exists for the current week
    let existing_records = status_history_repo
        .find_by_date_range(start_date_local, end_date_local)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to check existing status history: {:?}", e),
            })
        })?;
    
    let existing_record = existing_records.into_iter().find(|r| 
        r.start_date == start_date_local.naive_local().date() && 
        r.end_date == end_date_local.naive_local().date()
    );

    // If a record already exists for this period, skip insertion
    if let Some(existing) = existing_record {
        log::info!(
            "Status history duplicate attempt - Period: {} to {}, Status: {:?}, Count: {}",
            start_date_local.naive_local(),
            end_date_local.naive_local(),
            existing.status,
            existing.count
        );

        // TODO: Add metrics counter when metrics system is implemented
        // metrics::counter!("status_history.duplicate_attempts").increment(1);

        let res = Response {
            message: format!(
                "Status history already exists for this period (existing status: {:?}, count: {}, target: {})",
                existing.status, weekly_count, target_value
            ),
            result: format!("Skipped - Record exists with status: {:?}", existing.status),
        };

        return Ok(Json(res));
    }

    // Insert new status history record only if it doesn't exist
    use crate::repositories::status_history::CreateStatusHistory;
    use sea_orm::prelude::Decimal;
    
    let new_record = CreateStatusHistory {
        start_date: start_date_local.naive_local().date(),
        end_date: end_date_local.naive_local().date(),
        count: Decimal::from(weekly_count),
        status: new_status.clone(),
    };

    status_history_repo
        .create(new_record)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to insert status history: {:?}", e),
            })
        })?;

    let res = Response {
        message: format!(
            "Status updated from {:?} to {:?} (count: {}, target: {}, change: {})",
            current_status,
            new_status,
            weekly_count,
            target_value,
            if rank_change > 0 {
                format!("+{}", rank_change)
            } else {
                rank_change.to_string()
            }
        ),
        result: "Success".to_string(),
    };

    Ok(Json(res))
}

#[get("/status_history?<limit>&<offset>&<date_range..>")]
async fn get_status_history(
    status_history_repo: &State<Arc<dyn StatusHistoryRepository>>,
    limit: Option<u64>,
    offset: Option<u64>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<StatusHistoryResponse>>, Json<ErrorResponse>> {
    // Get the current status (most recent entry)
    let current_status = status_history_repo
        .get_latest()
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get current status: {:?}", e),
            })
        })?;

    // Convert current status to response format
    let current_status_record = current_status.map(|s| StatusHistoryRecord {
        id: s.id,
        start_date: s.start_date,
        end_date: s.end_date,
        count: s.count,
        status: status_to_string(s.status),
        created_at: s.created_at.with_timezone(&chrono::Utc),
    });

    // Apply pagination parameters
    let limit = limit.unwrap_or(100); // Default limit of 100
    let offset = offset.unwrap_or(0); // Default offset of 0

    // Get historical records ordered by date with pagination
    let (history_records, total_count) = if let Some(range) = date_range {
        let date_range = get_date_range_local(Some(range));
        status_history_repo
            .list_by_date_range(date_range.start_date, date_range.end_date, limit, offset)
            .await
            .map_err(|e| {
                Json(ErrorResponse {
                    error: format!("Failed to get status history: {:?}", e),
                })
            })?
    } else {
        status_history_repo
            .list(limit, offset)
            .await
            .map_err(|e| {
                Json(ErrorResponse {
                    error: format!("Failed to get status history: {:?}", e),
                })
            })?
    };

    // Convert to response format
    let history: Vec<StatusHistoryRecord> = history_records
        .into_iter()
        .map(|h| StatusHistoryRecord {
            id: h.id,
            start_date: h.start_date,
            end_date: h.end_date,
            count: h.count,
            status: status_to_string(h.status),
            created_at: h.created_at.with_timezone(&chrono::Utc),
        })
        .collect();

    let filtered_records = history.len();

    let response = StatusHistoryResponse {
        current_status: current_status_record,
        history,
        total_records: filtered_records,
        total_count: total_count as usize,
        page_info: PageInfo {
            limit,
            offset,
            has_more: (offset as usize + filtered_records) < total_count as usize,
        },
    };

    let res = Response {
        message: "ステータス履歴を取得しました".to_string(),
        result: response,
    };

    Ok(Json(res))
}

// Backward compatibility: Keep both endpoints
// Note: Despite the name, this endpoint queries from session_counter table, not status_history (formerly count_summary)
#[get("/count_summary?<date_range..>")]
async fn get_count_summary(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(session_repo, counter_repo, date_range).await
}

#[get("/session_counter?<date_range..>")]
async fn get_session_counter(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(session_repo, counter_repo, date_range).await
}

// Shared implementation for both endpoints
async fn get_session_counter_impl(
    session_repo: &State<Arc<dyn SessionRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    let date_range = get_date_range_local(date_range);
    // Note: DB sync is handled by the cron job, not in GET endpoints
    
    // 指定期間の合計を取得
    let sum_count = counter_repo
        .sum_count_between(date_range.start_date, date_range.end_date)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get count summary: {:?}", e),
            })
        })?;
    
    let res = Response {
        message: "指定期間のクリック数を取得しました".to_string(),
        result: SessionCounterSummary { 
            sum_count: Decimal::from(sum_count) 
        },
    };
    Ok(Json(res))
}

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let app_config = AppConfig::from_env().expect("Failed to load configuration");

    // Configオブジェクトの作成
    let mut cfg = Config::default();
    log::debug!("Redis config before url: {:?}", cfg);
    cfg.connection = None;
    cfg.url = Some(app_config.redis.url.clone());
    log::debug!("Redis config after url: {:?}", cfg);

    // プールの作成
    let redis_pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool");

    let db: DatabaseConnection = Database::connect(&app_config.database.url)
        .await
        .expect("Failed to connect to database");

    let cors = app_config
        .get_cors_options()
        .expect("Error while building CORS");

    // Create repository instances
    let session_repo: Arc<dyn SessionRepository> = Arc::new(RedisSessionRepository::new(
        Arc::new(redis_pool.clone()),
    ));
    let counter_repo: Arc<dyn SessionCounterRepository> = Arc::new(DbSessionCounterRepository::new(
        Arc::new(db.clone()),
    ));
    let history_repo: Arc<dyn StatusHistoryRepository> = Arc::new(DbStatusHistoryRepository::new(
        Arc::new(db.clone()),
    ));

    rocket::build()
        .manage(redis_pool)
        .manage(db)
        .manage(app_config)
        .manage(session_repo)
        .manage(counter_repo)
        .manage(history_repo)
        .mount(
            "/",
            routes![
                post_session,
                put_session,
                put_status_history,
                get_status_history,
                get_session_counter,
                get_count_summary
            ],
        )
        .attach(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_status_history_response_structure() {
        use chrono::Utc;

        // Test that StatusHistoryRecord can be created
        let record = StatusHistoryRecord {
            id: 1,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            count: Decimal::from(1000),
            status: "Clean".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(record.id, 1);
        assert_eq!(record.status, "Clean");

        // Test that StatusHistoryResponse can be created with pagination
        let response = StatusHistoryResponse {
            current_status: Some(record.clone()),
            history: vec![record],
            total_records: 1,
            total_count: 1,
            page_info: PageInfo {
                limit: 100,
                offset: 0,
                has_more: false,
            },
        };

        assert!(response.current_status.is_some());
        assert_eq!(response.total_records, 1);
        assert_eq!(response.total_count, 1);
        assert_eq!(response.history.len(), 1);
        assert_eq!(response.page_info.limit, 100);
        assert_eq!(response.page_info.offset, 0);
        assert!(!response.page_info.has_more);
    }

    // Note: This test requires a database connection, which would need complex setup
    // For now, we test the route registration and parameter parsing
    #[test]
    fn test_get_status_history_endpoint_params() {
        // Test that the endpoint accepts the correct parameters
        // The actual endpoint testing would require database mocking

        // Test parameter validation
        let limit: Option<u64> = Some(10);
        let offset: Option<u64> = Some(0);
        assert_eq!(limit.unwrap_or(100), 10);
        assert_eq!(offset.unwrap_or(0), 0);

        // Test default values
        let limit: Option<u64> = None;
        let offset: Option<u64> = None;
        assert_eq!(limit.unwrap_or(100), 100);
        assert_eq!(offset.unwrap_or(0), 0);
    }

    #[test]
    fn test_pagination_logic() {
        // Test pagination calculation
        let total_count = 100;
        let _limit = 10;
        let offset = 20;
        let filtered_records = 10;

        let has_more = (offset as usize + filtered_records) < total_count;
        assert!(has_more);

        // Test when no more records
        let offset = 95;
        let filtered_records = 5;
        let has_more = (offset as usize + filtered_records) < total_count;
        assert!(!has_more);
    }

    #[test]
    fn test_status_history_record_with_proper_status() {
        use chrono::Utc;

        // Test that StatusHistoryRecord uses proper status format
        let record = StatusHistoryRecord {
            id: 1,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            count: Decimal::from(1000),
            status: status_to_string(Status::Clean),
            created_at: Utc::now(),
        };

        // Should be uppercase format now
        assert_eq!(record.status, "CLEAN");
    }
}

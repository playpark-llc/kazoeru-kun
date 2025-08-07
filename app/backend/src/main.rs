mod sea_generated;

use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter},
    Config, Pool, Runtime,
};
use futures::stream::StreamExt;
use opt::{
    create_expiration, create_session_cookie_with_expiration, get_date_range_local,
    get_redis_connection, DateRange,
};
use rocket::{
    http::CookieJar,
    serde::{json::Json, Serialize},
    State,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use sea_generated::{prelude::{SessionCounter, StatusHistory}, session_counter, status_history, sea_orm_active_enums::Status};
use sea_orm::{
    prelude::Decimal, sea_query::OnConflict, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, FromQueryResult, QueryFilter, QuerySelect, Set, QueryOrder,
};
use std::env;
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday, TimeZone};
use chrono_tz::Asia::Tokyo;

#[derive(Serialize)]
struct SessionCount {
    count: u32,
}

#[derive(Serialize)]
struct Response<T> {
    message: String,
    result: T,
}
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[macro_use]
extern crate rocket;

#[derive(FromQueryResult, Debug, Serialize, Clone)]
struct SessionCounterSummary {
    sum_count: Decimal,
}

#[post("/session")]
async fn post_session(
    redis_pool: &State<Pool>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Response<SessionCount>>, Json<ErrorResponse>> {
    let session_id: String;
    let some_cookie = cookies.get("session");
    match some_cookie {
        Some(cookie) => {
            // クッキーの有効期限を取得
            log::debug!("Cookie: {}, expires: {:?}", cookie.value(), cookie.expires());
            session_id = cookie.value().to_string();
        }
        None => {
            let expiration = create_expiration();
            session_id = create_session_cookie_with_expiration(cookies, expiration);
        }
    }
    // redisの接続を取得
    let mut redis_con = get_redis_connection(redis_pool).await.map_err(|e| {
        Json(ErrorResponse {
            error: e,
        })
    })?;

    // redisにセッションIDを保存
    let session_count = redis_con.incr(session_id.clone(), 1).await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to incr session_id to redis: {:?}", e),
        })
    })?;

    let res = match session_count {
        // session_idのカウントが1の場合は有効期限を設定
        1 => {
            let expiration = create_expiration();
            redis_con
                .expire_at::<String, u32>(session_id.clone(), expiration.timestamp())
                .await
                .map_err(|e| {
                    Json(ErrorResponse {
                        error: format!("Failed to set expiration to redis: {:?}", e),
                    })
                })?;

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
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // redisの接続を取得
    let mut redis_con = get_redis_connection(redis_pool).await.map_err(|e| {
        Json(ErrorResponse {
            error: e,
        })
    })?;

    // セッションIDのイテレータ取得を保存
    let mut session_id_iter: AsyncIter<String> = redis_con.scan().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to keys * redis connection: {:?}", e),
        })
    })?;

    let mut insert_records = vec![];
    while let Some(session_id) = session_id_iter.next().await {
        let mut redis_con = get_redis_connection(redis_pool).await.map_err(|e| {
            Json(ErrorResponse {
                error: e,
            })
        })?;
        let insert_record = tokio::spawn(async move {
            let redis_count: i32 = redis_con.get(&session_id).await.unwrap_or(0);
            session_counter::ActiveModel {
                session_id: Set(session_id.clone().to_owned()),
                count: Set(redis_count.to_owned()),
                ..Default::default()
            }
        })
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to create insert record: {:?}", e),
            })
        })?;

        insert_records.push(insert_record);
    }

    if insert_records.len() == 0 {
        return Ok(Json(Response {
            message: "更新するDBのセッション情報はありませんでした".to_string(),
            result: "Success".to_string(),
        }));
    }
    let on_conflict = OnConflict::column(session_counter::Column::SessionId)
        .update_column(session_counter::Column::Count)
        .to_owned();

    // upsertの結果を取得
    let insert_result = SessionCounter::insert_many(insert_records)
        .on_conflict(on_conflict.clone())
        .exec(db.inner())
        .await;

    match insert_result {
        Ok(_) => {}
        Err(e) => {
            if e.eq(&DbErr::RecordNotInserted) {
                return Ok(Json(Response {
                    message: "更新するDBのセッション情報はありませんでした".to_string(),
                    result: "Success".to_string(),
                }));
            }
            return Err(Json(ErrorResponse {
                error: format!("Failed to upsert session_counter table in db: {:?}", e),
            }));
        }
    }
    let res = Response {
        message: "DBのセッション情報を更新しました".to_string(),
        result: "Success".to_string(),
    };
    Ok(Json(res))
}

// Helper function to get the current week range (Monday to Sunday) in Japan time
fn get_current_week_range() -> (chrono::DateTime<chrono_tz::Tz>, chrono::DateTime<chrono_tz::Tz>) {
    let now_jst = Tokyo.from_utc_datetime(&chrono::Utc::now().naive_utc());
    
    // Find the Monday of the current week
    let days_from_monday = match now_jst.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };
    
    let monday = now_jst - Duration::days(days_from_monday);
    let monday_start = Tokyo.from_local_datetime(
        &NaiveDate::from_ymd_opt(monday.year(), monday.month(), monday.day())
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    ).unwrap();
    
    let sunday_end = monday_start + Duration::days(7) - Duration::seconds(1);
    
    (monday_start, sunday_end)
}

// Helper function to calculate rank change based on count vs target
fn calculate_rank_change(count: i64, target: i64) -> i32 {
    if count >= target * 100 {
        3  // 100x target: 3 ranks up
    } else if count >= target * 10 {
        2  // 10x target: 2 ranks up
    } else if count >= target {
        1  // Above target: 1 rank up
    } else {
        -1 // Below target: 1 rank down
    }
}

// Helper function to apply rank change to status
fn apply_rank_change(current_status: Status, rank_change: i32) -> Status {
    let status_order = vec![
        Status::Dirty,
        Status::Unclean,
        Status::Clean,
        Status::Immaculate,
        Status::Spotless,
    ];
    
    let current_index = status_order.iter().position(|s| *s == current_status).unwrap_or(2); // Default to Clean
    let new_index = (current_index as i32 + rank_change).max(0).min(4) as usize;
    
    status_order[new_index].clone()
}

#[put("/status_history")]
async fn put_status_history(
    db: &State<DatabaseConnection>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // Get target value from environment variable (default: 100000)
    let target_value: i64 = env::var("TARGET_VALUE")
        .unwrap_or_else(|_| "100000".to_string())
        .parse()
        .unwrap_or(100000);
    
    // Get current week range in Japan time
    let (week_start, week_end) = get_current_week_range();
    
    // Convert to Local timezone for database query
    let start_date_local = Local.from_utc_datetime(&week_start.naive_utc());
    let end_date_local = Local.from_utc_datetime(&week_end.naive_utc());
    
    // Calculate the sum of session counts for the current week
    let session_count_result = SessionCounter::find()
        .filter(
            session_counter::Column::UpdatedAt.between(start_date_local, end_date_local),
        )
        .select_only()
        .column_as(session_counter::Column::Count.sum(), "sum_count")
        .into_model::<SessionCounterSummary>()
        .all(db.inner())
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to calculate weekly count: {:?}", e),
            })
        })?;
    
    let weekly_count = session_count_result
        .first()
        .and_then(|r| r.sum_count.to_string().parse::<i64>().ok())
        .unwrap_or(0);
    
    log::debug!("Weekly count: {}, Target: {}", weekly_count, target_value);
    
    // Get the current status from the database (most recent entry)
    let current_status_result = StatusHistory::find()
        .order_by_desc(status_history::Column::CreatedAt)
        .one(db.inner())
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
    
    log::debug!("Current status: {:?}, Rank change: {}, New status: {:?}", 
               current_status, rank_change, new_status);
    
    // Check if a record already exists for the current week
    let existing_record = StatusHistory::find()
        .filter(status_history::Column::StartDate.eq(start_date_local.naive_local()))
        .filter(status_history::Column::EndDate.eq(end_date_local.naive_local()))
        .one(db.inner())
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to check existing status history: {:?}", e),
            })
        })?;
    
    // If a record already exists for this period, skip insertion
    if existing_record.is_some() {
        log::debug!("Status history record already exists for period {} to {}", 
                   start_date_local.naive_local(), end_date_local.naive_local());
        
        let res = Response {
            message: format!(
                "Status history already exists for this period (count: {}, target: {})",
                weekly_count, target_value
            ),
            result: "Skipped".to_string(),
        };
        
        return Ok(Json(res));
    }
    
    // Insert new status history record only if it doesn't exist
    let new_record = status_history::ActiveModel {
        start_date: Set(start_date_local.naive_local()),
        end_date: Set(end_date_local.naive_local()),
        count: Set(Decimal::from(weekly_count)),
        status: Set(new_status.clone()),
        ..Default::default()
    };
    
    StatusHistory::insert(new_record)
        .exec(db.inner())
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to insert status history: {:?}", e),
            })
        })?;
    
    let res = Response {
        message: format!(
            "Status updated from {:?} to {:?} (count: {}, target: {}, change: {})",
            current_status, new_status, weekly_count, target_value, 
            if rank_change > 0 { format!("+{}", rank_change) } else { rank_change.to_string() }
        ),
        result: "Success".to_string(),
    };
    
    Ok(Json(res))
}

// Backward compatibility: Keep both endpoints
// Note: Despite the name, this endpoint queries from session_counter table, not status_history (formerly count_summary)
#[get("/count_summary?<date_range..>")]
async fn get_count_summary(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(redis_pool, db, date_range).await
}

#[get("/session_counter?<date_range..>")]
async fn get_session_counter(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    get_session_counter_impl(redis_pool, db, date_range).await
}

// Shared implementation for both endpoints
async fn get_session_counter_impl(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    let date_range = get_date_range_local(date_range);
    // DBを最新情報に更新
    let _ = put_session(redis_pool, db).await.map_err(|e| e);
    // upsertの結果を取得
    let session_count_summaries = SessionCounter::find()
        .filter(
            session_counter::Column::UpdatedAt.between(date_range.start_date, date_range.end_date),
        )
        .select_only()
        .column_as(session_counter::Column::Count.sum(), "sum_count")
        .into_model::<SessionCounterSummary>()
        .all(db.inner())
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to execute query: {:?}", e),
            })
        })?;

    log::debug!("session_count_summaries: {:?}", session_count_summaries);

    let res = Response {
        message: "指定期間のクリック数を取得しました".to_string(),
        result: session_count_summaries[0].clone(),
    };
    Ok(Json(res))
}

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();
    
    // Configオブジェクトの作成
    let mut cfg = Config::default();
    log::debug!("Redis config before url: {:?}", cfg);
    cfg.connection = None;
    cfg.url = Some(env::var("REDIS_URL")
        .expect("REDIS_URL environment variable must be set"));
    log::debug!("Redis config after url: {:?}", cfg);

    // プールの作成
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool");

    let db: DatabaseConnection = Database::connect(env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set"))
        .await
        .expect("Failed to connect to database");
    let allowed_origins: Vec<String> = env::var("ALLOWED_ORIGINS")
        .expect("ALLOWED_ORIGINS environment variable must be set")
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let exact = allowed_origins;
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&exact))
        .to_cors()
        .expect("Error while building CORS");

    rocket::build()
        .manage(redis_pool)
        .manage(db)
        .mount("/", routes![post_session, put_session, put_status_history, get_session_counter, get_count_summary])
        .attach(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    
    #[test]
    fn test_calculate_rank_change() {
        let target = 100000;
        
        // Test below target
        assert_eq!(calculate_rank_change(50000, target), -1);
        assert_eq!(calculate_rank_change(99999, target), -1);
        
        // Test at or above target (1 rank up)
        assert_eq!(calculate_rank_change(100000, target), 1);
        assert_eq!(calculate_rank_change(500000, target), 1);
        assert_eq!(calculate_rank_change(999999, target), 1);
        
        // Test 10x target (2 ranks up)
        assert_eq!(calculate_rank_change(1000000, target), 2);
        assert_eq!(calculate_rank_change(5000000, target), 2);
        assert_eq!(calculate_rank_change(9999999, target), 2);
        
        // Test 100x target (3 ranks up)
        assert_eq!(calculate_rank_change(10000000, target), 3);
        assert_eq!(calculate_rank_change(50000000, target), 3);
    }
    
    #[test]
    fn test_apply_rank_change() {
        // Test rank up from Clean
        assert_eq!(apply_rank_change(Status::Clean, 1), Status::Immaculate);
        assert_eq!(apply_rank_change(Status::Clean, 2), Status::Spotless);
        assert_eq!(apply_rank_change(Status::Clean, 3), Status::Spotless); // Capped at Spotless
        
        // Test rank down from Clean
        assert_eq!(apply_rank_change(Status::Clean, -1), Status::Unclean);
        assert_eq!(apply_rank_change(Status::Clean, -2), Status::Dirty);
        assert_eq!(apply_rank_change(Status::Clean, -3), Status::Dirty); // Capped at Dirty
        
        // Test boundary conditions
        assert_eq!(apply_rank_change(Status::Dirty, -1), Status::Dirty); // Can't go below Dirty
        assert_eq!(apply_rank_change(Status::Spotless, 1), Status::Spotless); // Can't go above Spotless
        
        // Test from other statuses
        assert_eq!(apply_rank_change(Status::Unclean, 1), Status::Clean);
        assert_eq!(apply_rank_change(Status::Immaculate, -1), Status::Clean);
    }
    
    #[test]
    fn test_get_current_week_range() {
        let (start, end) = get_current_week_range();
        
        // Check that start is a Monday at 00:00:00
        assert_eq!(start.weekday(), Weekday::Mon);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
        
        // Check that end is a Sunday at 23:59:59
        assert_eq!(end.weekday(), Weekday::Sun);
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
        
        // Check that the range is exactly 7 days
        let duration = end - start;
        assert_eq!(duration.num_days(), 6); // 6 full days + 23:59:59
    }
}

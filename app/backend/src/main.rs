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
use sea_generated::{prelude::SessionCounter, session_counter};
use sea_orm::{
    prelude::Decimal, sea_query::OnConflict, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, FromQueryResult, QueryFilter, QuerySelect, Set,
};
use std::env;

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
        .mount("/", routes![post_session, put_session, get_session_counter, get_count_summary])
        .attach(cors)
}

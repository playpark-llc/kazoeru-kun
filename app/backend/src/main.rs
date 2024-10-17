mod sea_generated;

use ::chrono::{DateTime, Datelike, Days, DurationRound, Local, NaiveDate, TimeDelta};
use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter},
    Config, Connection, Pool, Runtime,
};
use futures::stream::StreamExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::State;
use rocket::{
    form::FromForm,
    http::{Cookie, CookieJar},
    serde::{json::Json, Serialize},
    time::OffsetDateTime,
};
use sea_generated::{prelude::SessionCounter, session_counter};
use sea_orm::{
    prelude::Decimal,
    sea_query::OnConflict,
    sqlx::types::chrono::{self, TimeZone},
    ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect, Set,
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

// cookieの有効期限を取得
fn create_expiration() -> chrono::DateTime<Local> {
    // 現在の日付を取得
    let now = Local::now();

    // 今日の23:59:59を設定
    Local
        .with_ymd_and_hms(now.year(), now.month(), now.day() + 1, 0, 0, 0)
        .unwrap() // 有効な日時であることが確実なのでunwrapを使用
}

// 期限付きのsession_idをCookieに設定
fn create_session_cookie_with_expiration(
    cookies: &CookieJar<'_>,
    expiration: chrono::DateTime<Local>,
) -> String {
    let session_id = generate_session_id();

    // chronoのDateTimeをtime::OffsetDateTimeに変換
    let offset_expiration = OffsetDateTime::from_unix_timestamp(expiration.timestamp()).unwrap();

    // Cookieを作成し、有効期限を設定
    let mut cookie = Cookie::new("session", session_id.clone());
    cookie.set_expires(offset_expiration);

    println!("cookie value: {:?}", cookie.value());
    println!("cookie expires:{:?}", cookie.expires());

    cookies.add(cookie);

    session_id
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
            println!("{}, {:?}", cookie.value(), cookie.expires());
            session_id = cookie.value().to_string();
        }
        None => {
            let expiration = create_expiration();
            session_id = create_session_cookie_with_expiration(cookies, expiration);
        }
    }
    // redisの接続を取得
    let mut redis_con = get_redis_connection(redis_pool).await;

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
    let mut redis_con = get_redis_connection(redis_pool).await;

    // セッションIDのイテレータ取得を保存
    let mut session_id_iter: AsyncIter<String> = redis_con.scan().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to keys * redis connection: {:?}", e),
        })
    })?;

    let mut insert_records = vec![];
    while let Some(session_id) = session_id_iter.next().await {
        let mut redis_con = get_redis_connection(redis_pool).await;
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
        result: "Sccsess".to_string(),
    };
    Ok(Json(res))
}

#[derive(FromForm)]
struct DateRange {
    start_date: Option<rocket::time::Date>,
    end_date: Option<rocket::time::Date>,
}
#[get("/count_summary?<date_range..>")]
async fn get_count_summary(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<SessionCounterSummary>>, Json<ErrorResponse>> {
    // 期間の指定がない場合は、先週の月曜日から日曜日までの期間を取得
    let mut start_date: DateTime<Local> = Local::now()
        .duration_trunc(TimeDelta::days(1))
        .unwrap()
        .checked_sub_days(Days::new(
            (Local::now().weekday().number_from_monday() + 6) as u64,
        ))
        .unwrap();
    let mut end_date: DateTime<Local> = Local::now()
        .duration_trunc(TimeDelta::days(1))
        .unwrap()
        .checked_sub_days(Days::new(
            (Local::now().weekday().number_from_monday() - 1) as u64,
        ))
        .unwrap();
    if let Some(date_range) = date_range {
        if let Some(start_date_input) = date_range.start_date {
            println!("start_date_input: {:?}", start_date_input);
            let naive_start_date = NaiveDate::from_ymd_opt(
                start_date_input.year().into(),
                (start_date_input.month() as u32).into(),
                start_date_input.day().into(),
            )
            .unwrap();
            start_date = Local
                .from_local_datetime(&naive_start_date.and_hms_opt(0, 0, 0).unwrap())
                .single()
                .unwrap();
        }
        if let Some(end_date_input) = date_range.end_date {
            println!("end_date_input: {:?}", end_date_input);
            let naive_end_date = NaiveDate::from_ymd_opt(
                end_date_input.year().into(),
                (end_date_input.month() as u32).into(),
                end_date_input.day().into(),
            )
            .unwrap();
            end_date = Local
                .from_local_datetime(&naive_end_date.and_hms_opt(0, 0, 0).unwrap())
                .single()
                .unwrap();
        }
    };
    println!("start_date: {:?}", start_date);
    println!("end_date: {:?}", end_date);
    // DBを最新情報に更新
    let _ = put_session(redis_pool, db).await.map_err(|e| e);
    // upsertの結果を取得
    let session_count_summaries = SessionCounter::find()
        .filter(session_counter::Column::UpdatedAt.between(start_date, end_date))
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

    println!("session_count_summaries: {:?}", session_count_summaries);

    let res = Response {
        message: "指定期間のクリック数を取得しました".to_string(),
        result: session_count_summaries[0].clone(),
    };
    Ok(Json(res))
}

async fn get_redis_connection(redis_pool: &State<Pool>) -> Connection {
    match redis_pool.get().await {
        Ok(con) => con,
        Err(e) => panic!("Failed to get redis connection: {:?}", e),
    }
}

// ランダムなセッションIDを生成する関数
fn generate_session_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30) // 30文字のランダムな文字列を生成
        .map(char::from)
        .collect()
}

#[launch]
async fn rocket() -> _ {
    // Configオブジェクトの作成
    let mut cfg = Config::default();
    println!("cfg: {:?}", cfg);
    cfg.connection = None;
    cfg.url = Some(env::var("REDIS_URL").unwrap());
    println!("cfg: {:?}", cfg);

    // プールの作成
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    let db: DatabaseConnection = Database::connect(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    rocket::build()
        .manage(redis_pool)
        .manage(db)
        .mount("/", routes![post_session, put_session, get_count_summary])
}

mod sea_generated;

use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter},
    Config, Connection, Pool, Runtime,
};
use futures::stream::StreamExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{json::Json, Serialize};
use rocket::State;
use sea_generated::{prelude::SessionCounter, session_counter};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Database, DatabaseConnection, EntityTrait, NotSet,
    QueryFilter, Set,
};
use std::env;

#[derive(Serialize)]
struct SingleResult {
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

#[post("/session")]
async fn post_session(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Response<SingleResult>>, Json<ErrorResponse>> {
    let session_id: String;
    if let Some(session) = cookies.get("session") {
        session_id = session.value().to_string();
    } else {
        session_id = generate_session_id();
        // cookieをセット
        cookies.add(Cookie::new("session", session_id.clone()));
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
        // session_count=1の場合はSessionCounterテーブルにデータがないか確認
        1 => {
            // session_idが一致するデータを取得
            let session_counter_record = SessionCounter::find()
                .filter(session_counter::Column::SessionId.eq(&session_id))
                .one(db.inner())
                .await
                .map_err(|e| {
                    Json(ErrorResponse {
                        error: format!("Failed to execute query: {:?}", e),
                    })
                })?;

            match session_counter_record {
                // SessionCounterテーブルにsession_idが存在する場合
                // redisのカウントをSessionCounterのcount分インクリメント
                Some(session_counter) => {
                    // redisの接続を取得
                    let mut redis_con = get_redis_connection(redis_pool).await;
                    let session_count = redis_con
                        .incr(session_id.clone(), session_counter.count)
                        .await
                        .map_err(|e| {
                            Json(ErrorResponse {
                                error: format!("Failed to incr session_id to redis: {:?}", e),
                            })
                        })?;
                    Response {
                        message: "セッションカウントが更新されました".to_string(),
                        result: SingleResult {
                            count: session_count,
                        },
                    }
                }
                // SessionCounterテーブルにsession_idが存在しない場合
                // SessionCounterテーブルに新規データを登録
                None => {
                    let inserting_record = session_counter::ActiveModel {
                        id: NotSet,
                        session_id: Set(session_id),
                        count: Set(1),
                        created_at: NotSet,
                        updated_at: NotSet,
                    };
                    let _session_counter_created = SessionCounter::insert(inserting_record)
                        .exec(db.inner())
                        .await
                        .unwrap();

                    Response {
                        message: "新規セッションが登録されました".to_string(),
                        result: SingleResult { count: 1 },
                    }
                }
            }
        }
        // アクセス数が1以外の場合はSessionCounterテーブルを更新しない
        _ => Response {
            message: "セッションカウントが更新されました".to_string(),
            result: SingleResult {
                count: session_count,
            },
        },
    };
    return Ok(Json(res));
}

#[put("/session")]
async fn put_session(
    redis_pool: &State<Pool>,
    db: &State<DatabaseConnection>
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // redisの接続を取得
    let mut redis_con = get_redis_connection(redis_pool).await;

    // redisにセッションIDを保存
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

            println!("redis:{:?}", redis_count);

            session_counter::ActiveModel {
                session_id: Set(session_id.clone().to_owned()),
                count: Set(redis_count.to_owned()),
                ..Default::default()
            }
        })
        .await
        .unwrap();
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
    SessionCounter::insert_many(insert_records)
        .on_conflict(on_conflict.clone())
        .exec(db.inner())
        .await
        .unwrap();

    // redisのデータを削除
    let mut redis_con = get_redis_connection(redis_pool).await;
    deadpool_redis::redis::cmd("FLUSHDB")
        .query_async(&mut redis_con)
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to upsert session counter: {:?}", e),
            })
        })?;
    let res = Response {
        message: "DBのセッション情報を更新しました".to_string(),
        result: "Sccsess".to_string(),
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
        .mount("/", routes![post_session, put_session])
}

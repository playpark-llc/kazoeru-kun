use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter},
    Config, Connection, Pool, Runtime,
};
use futures::stream::StreamExt;
use prisma::PrismaClient;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{json::Json, Serialize};
use rocket::State;
use std::sync::Arc;
mod prisma;

#[derive(Serialize)]
struct SingleResult {
    session_id: String,
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
    prisma_client: &State<Arc<PrismaClient>>,
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
        // アクセス数が1の場合はSessionCounterテーブルに新規登録
        1 => {
            // session_idが一致するデータを取得
            let session_counter = prisma_client
                .session_counter()
                .find_unique(prisma::session_counter::session_id::equals(
                    session_id.clone(),
                ))
                .exec()
                .await
                .map_err(|e| {
                    Json(ErrorResponse {
                        error: format!("Failed to execute query: {:?}", e),
                    })
                })?;
            match session_counter {
                Some(session_counter) => Response {
                    message: "セッションカウントが更新されました".to_string(),
                    result: SingleResult {
                        session_id: session_counter.session_id,
                        count: session_counter.count as u32,
                    },
                },
                None => {
                    let session_counter = prisma_client
                        .session_counter()
                        .create(session_id.clone(), 1, vec![])
                        .exec()
                        .await
                        .map_err(|e| {
                            return Json(ErrorResponse {
                                error: format!("Failed to create session counter: {:?}", e),
                            });
                        })?;

                    Response {
                        message: "新規セッションが登録されました".to_string(),
                        result: SingleResult {
                            session_id: session_counter.session_id,
                            count: session_counter.count as u32,
                        },
                    }
                }
            }
        }
        // アクセス数が1以外の場合はSessionCounterテーブルを更新しない
        _ => Response {
            message: "セッションカウントが更新されました".to_string(),
            result: SingleResult {
                session_id: session_id.clone(),
                count: session_count,
            },
        },
    };
    return Ok(Json(res));
}

#[put("/session")]
async fn put_session(
    redis_pool: &State<Pool>,
    prisma_client: &State<Arc<PrismaClient>>,
) -> Result<Json<Response<Vec<SingleResult>>>, Json<ErrorResponse>> {
    // redisの接続を取得
    let mut redis_con = get_redis_connection(redis_pool).await;

    // redisにセッションIDを保存
    let mut session_id_iter: AsyncIter<String> = redis_con.scan().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to keys * redis connection: {:?}", e),
        })
    })?;

    let prisma_client = Arc::clone(&prisma_client);
    let session_counter_find_many = prisma_client
        .session_counter()
        .find_many(vec![prisma::session_counter::session_id::not(
            "".to_string(),
        )])
        .exec()
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to find many session counter: {:?}", e),
            })
        })?;

    let mut handles = vec![];
    while let Some(session_id) = session_id_iter.next().await {
        let mut redis_con = get_redis_connection(redis_pool).await;
        let prisma_client = Arc::clone(&prisma_client);
        let session_counter_find_many = session_counter_find_many.clone();
        let handle = tokio::spawn(async move {
            let redis_count: i32 = redis_con.get(&session_id).await.unwrap_or(0);

            let db_count = match session_counter_find_many
                .iter()
                .find(|&s| s.session_id == session_id.clone())
            {
                Some(s) => s.count,
                None => 0,
            };

            println!("redis:{:?}", redis_count);
            println!("db:{:?}", db_count);
            prisma_client
                .session_counter()
                .upsert(
                    prisma::session_counter::session_id::equals(session_id.clone()),
                    (session_id.clone(), redis_count + db_count, vec![]),
                    vec![prisma::session_counter::count::set(redis_count + db_count)],
                )
                .exec()
                .await
                .map_err(|e| {
                    Json(ErrorResponse {
                        error: format!("Failed to upsert session counter: {:?}", e),
                    })
                })
        });
        handles.push(handle);
    }
    // upsertの結果を取得
    let mut res_upsert_list = vec![];
    for handle in handles {
        let res = handle.await.unwrap()?; // エラーハンドリングの改善が必要
        res_upsert_list.push(SingleResult {
            session_id: res.session_id,
            count: res.count as u32,
        });
    }
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
        result: res_upsert_list,
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
    cfg.url = Some("redis://redis:6379/".to_string());
    println!("cfg: {:?}", cfg);

    // プールの作成
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    // prismaクライアントの初期化
    let prisma_client = Arc::from(PrismaClient::_builder().build().await.unwrap());

    rocket::build()
        .manage(redis_pool)
        .manage(prisma_client)
        .mount("/", routes![post_session, put_session])
}

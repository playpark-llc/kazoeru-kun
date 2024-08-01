use deadpool_redis::redis::{AsyncCommands, AsyncIter};
use deadpool_redis::{Config, Connection, Pool, Runtime};
use futures::stream::{self, Collect, StreamExt};
use prisma::PrismaClient;
use redis_async::client;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use std::env;
use std::sync::Arc;
use tokio::{sync::Mutex, task};
mod prisma;

#[derive(Deserialize)]
struct CreateRequest {
    session_id: String,
}
#[derive(Serialize)]
struct SingleResult {
    session_id: String,
    count: u32,
}
#[derive(Serialize)]
struct SingleResponse {
    message: String,
    result: SingleResult,
}
#[derive(Serialize)]
struct MultiResponse {
    message: String,
    result: Vec<prisma::session_counter::Data>,
}
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
#[macro_use]
extern crate rocket;

#[post("/session", data = "<input>")]
async fn post_session(
    redis_pool: &State<Pool>,
    prisma_client: &State<Arc<PrismaClient>>,
    input: Json<CreateRequest>,
) -> Result<Json<SingleResponse>, Json<ErrorResponse>> {
    // redisの接続を取得
    let mut redis_con = redis_pool.get().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to get redis connection: {:?}", e),
        })
    })?;

    // redisにセッションIDを保存
    let session_count = redis_con
        .incr(input.session_id.clone(), 1)
        .await
        .map_err(|e| {
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
                    input.session_id.clone(),
                ))
                .exec()
                .await
                .map_err(|e| {
                    Json(ErrorResponse {
                        error: format!("Failed to execute query: {:?}", e),
                    })
                })?;
            match session_counter {
                Some(session_counter) => SingleResponse {
                    message: format!(
                        "session_id: {:?}のアクセス数は{:?}です",
                        input.session_id.clone(),
                        session_count
                    ),
                    result: SingleResult {
                        session_id: session_counter.session_id,
                        count: session_counter.count as u32,
                    },
                },
                None => {
                    let session_counter = prisma_client
                        .session_counter()
                        .create(input.session_id.clone(), 1, vec![])
                        .exec()
                        .await
                        .map_err(|e| {
                            return Json(ErrorResponse {
                                error: format!("Failed to create session counter: {:?}", e),
                            });
                        })?;

                    SingleResponse {
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
        _ => SingleResponse {
            message: format!(
                "session_id: {:?}のアクセス数は{:?}です",
                input.session_id.clone(),
                session_count
            ),
            result: SingleResult {
                session_id: input.session_id.clone(),
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
) -> Result<Json<SingleResponse>, Json<ErrorResponse>> {
    // redisの接続を取得
    let mut redis_con = redis_pool.get().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to get redis connection: {:?}", e),
        })
    })?;

    // redisにセッションIDを保存
    let mut session_id_iter: AsyncIter<String> = redis_con.scan().await.map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to keys * redis connection: {:?}", e),
        })
    })?;

    let mut handles = vec![];
    while let Some(session_id) = session_id_iter.next().await {
        let mut redis_con = redis_pool.get().await.map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get redis connection: {:?}", e),
            })
        })?;
        let prisma_client = Arc::clone(&prisma_client);
        let handle = tokio::spawn(async move {
            let count: i32 = redis_con.get(&session_id).await.unwrap_or(0);
            prisma_client
                .session_counter()
                .upsert(
                    prisma::session_counter::session_id::equals(session_id.clone()),
                    (session_id.clone(), count, vec![]),
                    vec![prisma::session_counter::count::set(count)],
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

    for handle in handles {
        handle.await.unwrap(); // エラーハンドリングの改善が必要
    }

    let res = SingleResponse {
        message: "セッション情報を更新しました".to_string(),
        result: SingleResult {
            session_id: "test".to_string(),
            count: 0,
        },
    };
    Ok(Json(res))
}

#[launch]
async fn rocket() -> _ {
    // Configオブジェクトの作成
    let mut cfg = Config::default();
    println!("cfg: {:?}",cfg);
    cfg.connection = None;
    cfg.url = Some("redis://redis:6379/".to_string());
    println!("cfg: {:?}",cfg);

    // プールの作成
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();


    // prismaクライアントの初期化
    let prisma_client = Arc::from(PrismaClient::_builder().build().await.unwrap());

    rocket::build()
        .manage(redis_pool)
        .manage(prisma_client)
        .mount("/", routes![post_session, put_session])
}

use async_std::task;
use tide::{Request, Server};
use redis::{AsyncCommands, Client};
use std::time::{SystemTime, UNIX_EPOCH};

async fn count_records(req: Request<Client>) -> tide::Result<String>{
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    let redis_client = req.state();
    let mut con = redis_client.get_async_connection().await.unwrap();

    let _: () = con.zadd("timestamps", now as f64, now as f64).await.unwrap();

    let one_second_ago = now -1000;

    let count: isize = con.zcount("timestamps", one_second_ago as f64, now as f64).await.unwrap();

    Ok(format!("Number of records in the last second: {}", count))
}

async fn run_server(redis_client: Client) -> tide::Result<()>{
    let mut app = Server::with_state(redis_client);
    app.at("/").get(count_records);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

fn main() {
    let redis_client = Client::open("redis://redis:6379/").unwrap();
    task::block_on(run_server(redis_client));
}

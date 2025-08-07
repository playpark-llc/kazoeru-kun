use opt::config::AppConfig;
use opt::repositories::{
    RedisSessionRepository, SessionRepository,
    DbSessionCounterRepository, SessionCounterRepository,
    DbStatusHistoryRepository, StatusHistoryRepository,
};
use opt::routes::configure_routes;
use deadpool_redis::{
    Config, Runtime,
};
use std::sync::Arc;
use sea_orm::{Database, DatabaseConnection};

#[macro_use]
extern crate rocket;






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
        .mount("/", configure_routes())
        .attach(cors)
}

// Tests are now located in the respective handler modules

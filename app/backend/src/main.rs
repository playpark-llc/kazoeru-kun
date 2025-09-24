use opt::config::AppConfig;
use opt::di::DiContainer;
use opt::routes::configure_routes;
use std::time::Duration;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();
    log::info!("Starting Kazoeru-kun application...");

    // Load configuration
    let app_config = AppConfig::from_env().expect("Failed to load configuration");

    // Build dependency injection container
    let container = DiContainer::build(app_config)
        .await
        .expect("Failed to build dependency injection container");

    // Start cron job for syncing Redis to Database
    spawn_sync_cron_job(container.sync_service.clone());

    // Get CORS configuration
    let cors = container.get_cors().expect("Failed to configure CORS");

    // Build and configure Rocket application
    rocket::build()
        // Manage services for handler injection
        .manage(container.session_service.clone())
        .manage(container.status_service.clone())
        .manage(container.sync_service.clone())
        .manage(container.health_service.clone())
        // Manage repositories for direct access if needed
        .manage(container.session_repo.clone())
        .manage(container.counter_repo.clone())
        .manage(container.history_repo.clone())
        // Manage infrastructure components
        .manage(container.redis_pool.clone())
        .manage(container.db.clone())
        .manage(container.config.clone())
        // Mount routes and attach CORS
        .mount("/", configure_routes())
        .attach(cors)
}

/// Spawn a background task to sync Redis sessions to database periodically
fn spawn_sync_cron_job(sync_service: std::sync::Arc<opt::services::SyncService>) {
    tokio::spawn(async move {
        // Create an interval timer for 5 minutes (300 seconds)
        let mut interval = tokio::time::interval(Duration::from_secs(300));

        log::info!("Starting cron job for Redis to Database sync (5 minute intervals)");

        loop {
            interval.tick().await;

            log::info!("Running scheduled sync from Redis to Database...");

            match sync_service.sync_sessions_to_database().await {
                Ok(result) => {
                    log::info!(
                        "Sync completed: {} sessions synced, message: {}",
                        result.synced_count,
                        result.message
                    );
                }
                Err(e) => {
                    log::error!("Failed to sync sessions: {:?}", e);
                }
            }
        }
    });
}

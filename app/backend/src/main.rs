use opt::di::DiContainer;
use opt::config::AppConfig;
use opt::routes::configure_routes;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();
    log::info!("Starting Kazoeru-kun application...");

    // Load configuration
    let app_config = AppConfig::from_env()
        .expect("Failed to load configuration");

    // Build dependency injection container
    let container = DiContainer::build(app_config)
        .await
        .expect("Failed to build dependency injection container");

    // Get CORS configuration
    let cors = container.get_cors()
        .expect("Failed to configure CORS");

    // Build and configure Rocket application
    rocket::build()
        // Manage services for handler injection
        .manage(container.session_service.clone())
        .manage(container.status_service.clone())
        .manage(container.sync_service.clone())
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
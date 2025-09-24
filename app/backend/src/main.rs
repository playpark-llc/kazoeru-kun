use chrono::TimeZone;
use opt::config::AppConfig;
use opt::di::DiContainer;
use opt::routes::configure_routes;
use std::future::Future;
use std::sync::Arc;
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

    // Start cron job for updating status history
    spawn_status_history_cron_job(container.status_service.clone(), Arc::new(container.config.clone()));

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

/// Configuration for a cron job
struct CronJobConfig {
    name: &'static str,
    interval_secs: u64,
    initial_delay: Option<Duration>,
    max_retries: u32,
    retry_delay_secs: u64,
}

/// Generic function to spawn a cron job with retry logic
fn spawn_cron_job<F, Fut>(config: CronJobConfig, task: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    tokio::spawn(async move {
        // Handle initial delay if specified
        if let Some(delay) = config.initial_delay {
            log::info!(
                "Starting cron job '{}' with initial delay: {:?}",
                config.name,
                delay
            );
            tokio::time::sleep(delay).await;
        } else {
            log::info!(
                "Starting cron job '{}' (interval: {} seconds)",
                config.name,
                config.interval_secs
            );
        }

        // Create interval timer
        let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));

        loop {
            interval.tick().await;
            log::info!("Running scheduled task: {}", config.name);

            let mut retry_count = 0;
            loop {
                match task().await {
                    Ok(message) => {
                        log::info!("{}: {}", config.name, message);
                        break;
                    }
                    Err(e) if retry_count < config.max_retries => {
                        retry_count += 1;
                        log::warn!(
                            "{} failed (attempt {}/{}): {:?}",
                            config.name,
                            retry_count,
                            config.max_retries,
                            e
                        );
                        tokio::time::sleep(Duration::from_secs(config.retry_delay_secs)).await;
                    }
                    Err(e) => {
                        log::error!(
                            "{} failed after {} attempts: {:?}",
                            config.name,
                            config.max_retries,
                            e
                        );
                        // TODO: Send metrics/alerts for monitoring
                        break;
                    }
                }
            }
        }
    });
}

/// Spawn a background task to sync Redis sessions to database periodically
fn spawn_sync_cron_job(sync_service: std::sync::Arc<opt::services::SyncService>) {
    let sync_service_clone = sync_service.clone();

    spawn_cron_job(
        CronJobConfig {
            name: "Redis to Database sync",
            interval_secs: 300, // 5 minutes
            initial_delay: None,
            max_retries: 3,
            retry_delay_secs: 30,
        },
        move || {
            let service = sync_service_clone.clone();
            async move {
                service
                    .sync_sessions_to_database()
                    .await
                    .map(|result| {
                        format!(
                            "Sync completed: {} sessions synced, message: {}",
                            result.synced_count, result.message
                        )
                    })
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
        },
    );
}

/// Spawn a background task to update status history daily at midnight
fn spawn_status_history_cron_job(
    status_service: std::sync::Arc<opt::services::StatusService>,
    config: std::sync::Arc<opt::config::AppConfig>,
) {
    // Calculate initial delay to next midnight
    let now = chrono::Local::now();
    let tomorrow_midnight = (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .expect("Failed to create midnight time");
    let tomorrow_midnight = chrono::Local.from_local_datetime(&tomorrow_midnight).unwrap();
    let initial_delay = (tomorrow_midnight - now)
        .to_std()
        .unwrap_or(Duration::from_secs(0));

    let status_service_clone = status_service.clone();
    let config_clone = config.clone();

    spawn_cron_job(
        CronJobConfig {
            name: "Status history update",
            interval_secs: 86400, // 24 hours
            initial_delay: Some(initial_delay),
            max_retries: 3,
            retry_delay_secs: 60,
        },
        move || {
            let service = status_service_clone.clone();
            let cfg = config_clone.clone();
            async move {
                service
                    .update_weekly_status(cfg.target_value)
                    .await
                    .map(|result| {
                        format!(
                            "Status history update completed: {}, status changed from {} to {}",
                            result.message, result.previous_status, result.new_status
                        )
                    })
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
        },
    );
}
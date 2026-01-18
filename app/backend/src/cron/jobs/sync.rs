use crate::cron::{config::CronJobConfig, executor::spawn_cron_job};
use std::sync::Arc;

/// Spawn a background task to sync Redis sessions to database periodically
pub fn spawn_sync_cron_job(sync_service: Arc<opt::services::SyncService>) {
    let sync_service_clone = sync_service.clone();

    let interval_secs = std::env::var("SYNC_JOB_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60); // Default: 1 minute

    let config = CronJobConfig::new("Redis to Database sync", interval_secs)
        .with_retries(3, 30);

    spawn_cron_job(config, move || {
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
    });
}
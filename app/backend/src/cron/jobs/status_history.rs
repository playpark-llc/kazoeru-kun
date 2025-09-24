use crate::cron::{config::CronJobConfig, executor::spawn_cron_job};
use chrono::{Local, TimeZone};
use std::sync::Arc;
use std::time::Duration;

/// Calculate initial delay to next midnight
fn calculate_delay_to_midnight() -> Duration {
    let now = Local::now();
    let tomorrow_midnight = (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .expect("Failed to create midnight time");
    let tomorrow_midnight = Local
        .from_local_datetime(&tomorrow_midnight)
        .unwrap();

    (tomorrow_midnight - now)
        .to_std()
        .unwrap_or(Duration::from_secs(0))
}

/// Spawn a background task to update status history daily at midnight
pub fn spawn_status_history_cron_job(
    status_service: Arc<opt::services::StatusService>,
    config: Arc<opt::config::AppConfig>,
) {
    let initial_delay = calculate_delay_to_midnight();
    let status_service_clone = status_service.clone();
    let config_clone = config.clone();

    let cron_config = CronJobConfig::new("Status history update", 86400) // 24 hours
        .with_initial_delay(initial_delay)
        .with_retries(3, 60);

    spawn_cron_job(cron_config, move || {
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
    });
}
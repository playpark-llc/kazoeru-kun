use super::config::CronJobConfig;
use std::future::Future;
use std::time::Duration;

/// Generic function to spawn a cron job with retry logic
pub fn spawn_cron_job<F, Fut>(config: CronJobConfig, task: F)
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
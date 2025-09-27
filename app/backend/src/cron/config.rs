use std::time::Duration;

/// Configuration for a cron job
#[derive(Debug, Clone)]
pub struct CronJobConfig {
    pub name: &'static str,
    pub interval_secs: u64,
    pub initial_delay: Option<Duration>,
    pub max_retries: u32,
    pub retry_delay_secs: u64,
}

impl CronJobConfig {
    pub fn new(name: &'static str, interval_secs: u64) -> Self {
        Self {
            name,
            interval_secs,
            initial_delay: None,
            max_retries: 3,
            retry_delay_secs: 30,
        }
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = Some(delay);
        self
    }

    pub fn with_retries(mut self, max_retries: u32, retry_delay_secs: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_secs = retry_delay_secs;
        self
    }
}
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            exponential_base: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
}

pub async fn retry_with_config<F, T, E, Fut>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) if attempt >= config.max_attempts => {
                log::error!(
                    "Operation failed after {} attempts: {:?}",
                    config.max_attempts,
                    err
                );
                return Err(err);
            }
            Err(err) => {
                log::warn!(
                    "Operation failed (attempt {}/{}): {:?}, retrying in {:?}",
                    attempt,
                    config.max_attempts,
                    err,
                    delay
                );
                
                sleep(delay).await;
                
                // Calculate next delay with exponential backoff
                delay = calculate_next_delay(
                    delay,
                    config.exponential_base,
                    config.max_delay,
                    config.jitter,
                );
            }
        }
    }
}

fn calculate_next_delay(
    current: Duration,
    exponential_base: f64,
    max_delay: Duration,
    jitter: bool,
) -> Duration {
    let mut next = Duration::from_secs_f64(current.as_secs_f64() * exponential_base);
    
    if next > max_delay {
        next = max_delay;
    }
    
    if jitter {
        // Add jitter up to 25% of the delay
        let jitter_amount = rand::random::<f64>() * 0.25;
        next = Duration::from_secs_f64(next.as_secs_f64() * (1.0 + jitter_amount));
    }
    
    next
}

// Convenience function for simple retry with default config
pub async fn retry<F, T, E, Fut>(operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    retry_with_config(RetryConfig::default(), operation).await
}

// Retry specifically for Redis operations
pub async fn retry_redis<F, T, Fut>(operation: F) -> Result<T, crate::repositories::RepositoryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, crate::repositories::RepositoryError>>,
{
    let config = RetryConfig::default()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(50));
    
    retry_with_config(config, operation).await
}

// Retry specifically for database operations
pub async fn retry_db<F, T, Fut>(operation: F) -> Result<T, crate::repositories::RepositoryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, crate::repositories::RepositoryError>>,
{
    let config = RetryConfig::default()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(100));
    
    retry_with_config(config, operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let count_clone = attempt_count.clone();
        
        let result = retry(|| {
            let count = count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err("First attempt fails")
                } else {
                    Ok("Success")
                }
            }
        }).await;
        
        assert_eq!(result, Ok("Success"));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let count_clone = attempt_count.clone();
        
        let result = retry(|| {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<String, &str>("Always fails")
            }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // Default max attempts
    }

    #[tokio::test]
    async fn test_retry_immediate_success() {
        let result = retry(|| async { Ok::<_, &str>("Immediate success") }).await;
        assert_eq!(result, Ok("Immediate success"));
    }

    #[tokio::test]
    async fn test_retry_with_custom_config() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let count_clone = attempt_count.clone();
        
        let config = RetryConfig::default()
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(10));
        
        let result = retry_with_config(config, || {
            let count = count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 3 {
                    Err("Fails first 3 attempts")
                } else {
                    Ok("Success on 4th attempt")
                }
            }
        }).await;
        
        assert_eq!(result, Ok("Success on 4th attempt"));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_retry_timing() {
        let start = std::time::Instant::now();
        let attempt_count = Arc::new(AtomicU32::new(0));
        let count_clone = attempt_count.clone();
        
        let config = RetryConfig::default()
            .with_max_attempts(3)
            .with_initial_delay(Duration::from_millis(50))
            .with_jitter(false);
        
        let _ = retry_with_config(config, || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<(), &str>("Always fails")
            }
        }).await;
        
        let elapsed = start.elapsed();
        // Should have delays of 50ms and 100ms (exponential backoff)
        assert!(elapsed >= Duration::from_millis(150));
        assert!(elapsed < Duration::from_millis(300));
    }

    #[tokio::test]
    async fn test_calculate_next_delay() {
        let current = Duration::from_millis(100);
        let next = calculate_next_delay(current, 2.0, Duration::from_secs(5), false);
        assert_eq!(next, Duration::from_millis(200));
        
        // Test max delay capping
        let current = Duration::from_secs(3);
        let next = calculate_next_delay(current, 2.0, Duration::from_secs(5), false);
        assert_eq!(next, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_calculate_next_delay_with_jitter() {
        let current = Duration::from_millis(100);
        let next = calculate_next_delay(current, 2.0, Duration::from_secs(5), true);
        
        // With jitter, the delay should be between 200ms and 250ms (up to 25% jitter)
        assert!(next >= Duration::from_millis(200));
        assert!(next <= Duration::from_millis(250));
    }
}
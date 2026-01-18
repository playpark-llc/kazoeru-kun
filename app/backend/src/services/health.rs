use crate::handlers::health_handler::{ComponentStatus, HealthResponse, ServiceStatus};
use deadpool_redis::{redis, Pool};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct HealthService {
    db: Arc<DatabaseConnection>,
    redis_pool: Arc<Pool>,
}

impl HealthService {
    pub fn new(db: Arc<DatabaseConnection>, redis_pool: Arc<Pool>) -> Self {
        Self { db, redis_pool }
    }

    pub async fn check_health(&self) -> Result<HealthResponse, String> {
        let database_status = self.check_database().await;
        let redis_status = self.check_redis().await;

        let overall_healthy =
            database_status.status == "healthy" && redis_status.status == "healthy";

        Ok(HealthResponse {
            status: if overall_healthy {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            components: ComponentStatus {
                database: database_status,
                redis: redis_status,
            },
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub async fn check_readiness(&self) -> Result<HealthResponse, String> {
        // For readiness, we check if all components are ready to serve traffic
        let health = self.check_health().await?;

        if health.status == "healthy" {
            Ok(health)
        } else {
            Err("Service not ready".to_string())
        }
    }

    async fn check_database(&self) -> ServiceStatus {
        let start = Instant::now();

        match sea_orm::ConnectionTrait::execute_unprepared(&*self.db, "SELECT 1").await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                ServiceStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(latency),
                    error: None,
                }
            }
            Err(e) => ServiceStatus {
                status: "unhealthy".to_string(),
                latency_ms: None,
                error: Some(format!("Database connection error: {e}")),
            },
        }
    }

    async fn check_redis(&self) -> ServiceStatus {
        let start = Instant::now();

        match self.redis_pool.get().await {
            Ok(mut conn) => {
                let result: redis::RedisResult<String> =
                    redis::cmd("PING").query_async(&mut conn).await;

                match result {
                    Ok(response) if response == "PONG" => {
                        let latency = start.elapsed().as_millis() as u64;
                        ServiceStatus {
                            status: "healthy".to_string(),
                            latency_ms: Some(latency),
                            error: None,
                        }
                    }
                    Ok(_) => ServiceStatus {
                        status: "unhealthy".to_string(),
                        latency_ms: None,
                        error: Some("Unexpected Redis response".to_string()),
                    },
                    Err(e) => ServiceStatus {
                        status: "unhealthy".to_string(),
                        latency_ms: None,
                        error: Some(format!("Redis ping failed: {e}")),
                    },
                }
            }
            Err(e) => ServiceStatus {
                status: "unhealthy".to_string(),
                latency_ms: None,
                error: Some(format!("Redis connection pool error: {e}")),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_response_structure() {
        // This is a basic test to ensure the response structure is correct
        let response = HealthResponse {
            status: "healthy".to_string(),
            components: ComponentStatus {
                database: ServiceStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(5),
                    error: None,
                },
                redis: ServiceStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(2),
                    error: None,
                },
            },
            timestamp: 1234567890,
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.components.database.status, "healthy");
        assert_eq!(response.components.redis.status, "healthy");
    }
}

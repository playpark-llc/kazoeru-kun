use rocket::{State, http::Status, get};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::services::health::HealthService;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub components: ComponentStatus,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub database: ServiceStatus,
    pub redis: ServiceStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub status: String,
    pub timestamp: i64,
}

#[get("/health")]
pub async fn health_check(
    health_service: &State<Arc<HealthService>>
) -> Result<Json<HealthResponse>, Status> {
    match health_service.check_health().await {
        Ok(response) => Ok(Json(response)),
        Err(_) => {
            // Return 503 Service Unavailable if any component is unhealthy
            Err(Status::ServiceUnavailable)
        }
    }
}

#[get("/health/liveness")]
pub async fn liveness_probe() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "alive".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

#[get("/health/readiness")]
pub async fn readiness_probe(
    health_service: &State<Arc<HealthService>>
) -> Result<Json<HealthResponse>, Status> {
    match health_service.check_readiness().await {
        Ok(response) => Ok(Json(response)),
        Err(_) => {
            // Return 503 Service Unavailable if not ready
            Err(Status::ServiceUnavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::{routes, Build, Rocket};

    fn create_test_rocket() -> Rocket<Build> {
        // This is a simplified test setup
        // In real tests, you'd need to properly mock the HealthService
        rocket::build()
            .mount("/", routes![liveness_probe])
    }

    #[test]
    fn test_liveness_probe() {
        let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
        let response = client.get("/health/liveness").dispatch();
        
        assert_eq!(response.status(), rocket::http::Status::Ok);
        
        let body = response.into_string().unwrap();
        assert!(body.contains("alive"));
    }
}
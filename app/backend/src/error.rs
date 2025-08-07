use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_code: String,
    pub details: Option<ErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
    pub retry_after: Option<u64>, // Seconds to wait before retry
}

#[derive(Debug)]
pub enum AppError {
    // Session-related errors
    SessionNotFound(String),
    SessionExpired(String),
    SessionInvalid(String),
    
    // Redis-related errors
    RedisConnectionError(String),
    RedisOperationError(String),
    RedisTimeout(String),
    
    // Database-related errors
    DatabaseConnectionError(String),
    DatabaseOperationError(String),
    DatabaseConstraintViolation(String),
    DatabaseTransactionError(String),
    
    // Configuration errors
    ConfigurationError(String),
    
    // Validation errors
    ValidationError(String),
    
    // Rate limiting
    RateLimitExceeded(u64), // Retry after seconds
    
    // Internal errors
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::SessionNotFound(msg) => write!(f, "Session not found: {}", msg),
            AppError::SessionExpired(msg) => write!(f, "Session expired: {}", msg),
            AppError::SessionInvalid(msg) => write!(f, "Invalid session: {}", msg),
            AppError::RedisConnectionError(msg) => write!(f, "Redis connection error: {}", msg),
            AppError::RedisOperationError(msg) => write!(f, "Redis operation error: {}", msg),
            AppError::RedisTimeout(msg) => write!(f, "Redis timeout: {}", msg),
            AppError::DatabaseConnectionError(msg) => write!(f, "Database connection error: {}", msg),
            AppError::DatabaseOperationError(msg) => write!(f, "Database operation error: {}", msg),
            AppError::DatabaseConstraintViolation(msg) => write!(f, "Database constraint violation: {}", msg),
            AppError::DatabaseTransactionError(msg) => write!(f, "Database transaction error: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::RateLimitExceeded(retry_after) => write!(f, "Rate limit exceeded. Retry after {} seconds", retry_after),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::SessionNotFound(_) => "SESSION_NOT_FOUND",
            AppError::SessionExpired(_) => "SESSION_EXPIRED",
            AppError::SessionInvalid(_) => "SESSION_INVALID",
            AppError::RedisConnectionError(_) => "REDIS_CONNECTION_ERROR",
            AppError::RedisOperationError(_) => "REDIS_OPERATION_ERROR",
            AppError::RedisTimeout(_) => "REDIS_TIMEOUT",
            AppError::DatabaseConnectionError(_) => "DATABASE_CONNECTION_ERROR",
            AppError::DatabaseOperationError(_) => "DATABASE_OPERATION_ERROR",
            AppError::DatabaseConstraintViolation(_) => "DATABASE_CONSTRAINT_VIOLATION",
            AppError::DatabaseTransactionError(_) => "DATABASE_TRANSACTION_ERROR",
            AppError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    pub fn status_code(&self) -> Status {
        match self {
            AppError::SessionNotFound(_) => Status::NotFound,
            AppError::SessionExpired(_) => Status::Unauthorized,
            AppError::SessionInvalid(_) => Status::BadRequest,
            AppError::RedisConnectionError(_) => Status::ServiceUnavailable,
            AppError::RedisOperationError(_) => Status::InternalServerError,
            AppError::RedisTimeout(_) => Status::RequestTimeout,
            AppError::DatabaseConnectionError(_) => Status::ServiceUnavailable,
            AppError::DatabaseOperationError(_) => Status::InternalServerError,
            AppError::DatabaseConstraintViolation(_) => Status::Conflict,
            AppError::DatabaseTransactionError(_) => Status::InternalServerError,
            AppError::ConfigurationError(_) => Status::InternalServerError,
            AppError::ValidationError(_) => Status::BadRequest,
            AppError::RateLimitExceeded(_) => Status::TooManyRequests,
            AppError::InternalError(_) => Status::InternalServerError,
        }
    }

    pub fn to_response(&self) -> ErrorResponse {
        let retry_after = match self {
            AppError::RateLimitExceeded(seconds) => Some(*seconds),
            AppError::RedisTimeout(_) => Some(5),
            _ => None,
        };

        ErrorResponse {
            error: self.to_string(),
            error_code: self.error_code().to_string(),
            details: Some(ErrorDetails {
                error_type: self.error_code().to_string(),
                timestamp: chrono::Utc::now(),
                request_id: None, // Can be enhanced with request tracking
                retry_after,
            }),
        }
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> response::Result<'static> {
        let status = self.status_code();
        let response = self.to_response();
        
        // Log the error
        match &self {
            AppError::InternalError(_) | 
            AppError::DatabaseOperationError(_) | 
            AppError::RedisOperationError(_) => {
                log::error!("Error occurred: {:?}", self);
            }
            AppError::RateLimitExceeded(_) => {
                log::warn!("Rate limit exceeded from {:?}", request.remote());
            }
            _ => {
                log::debug!("Error occurred: {:?}", self);
            }
        }
        
        response::Response::build_from(Json(response).respond_to(request)?)
            .status(status)
            .ok()
    }
}

// Conversion from repository errors
impl From<crate::repositories::RepositoryError> for AppError {
    fn from(err: crate::repositories::RepositoryError) -> Self {
        match err {
            crate::repositories::RepositoryError::DatabaseError(msg) => {
                if msg.contains("constraint") || msg.contains("duplicate") {
                    AppError::DatabaseConstraintViolation(msg)
                } else if msg.contains("transaction") {
                    AppError::DatabaseTransactionError(msg)
                } else {
                    AppError::DatabaseOperationError(msg)
                }
            }
            crate::repositories::RepositoryError::RedisError(msg) => {
                if msg.contains("connection") || msg.contains("pool") {
                    AppError::RedisConnectionError(msg)
                } else if msg.contains("timeout") {
                    AppError::RedisTimeout(msg)
                } else {
                    AppError::RedisOperationError(msg)
                }
            }
            crate::repositories::RepositoryError::NotFound => {
                AppError::SessionNotFound("Requested resource not found".to_string())
            }
            crate::repositories::RepositoryError::ConfigError(err) => {
                AppError::ConfigurationError(err.to_string())
            }
            crate::repositories::RepositoryError::TransactionError(msg) => {
                AppError::DatabaseTransactionError(msg)
            }
        }
    }
}

// Helper function for creating validation errors
pub fn validation_error(field: &str, message: &str) -> AppError {
    AppError::ValidationError(format!("Field '{}': {}", field, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = AppError::SessionExpired("test".to_string());
        assert_eq!(err.error_code(), "SESSION_EXPIRED");
        
        let err = AppError::RateLimitExceeded(60);
        assert_eq!(err.error_code(), "RATE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_status_codes() {
        let err = AppError::SessionNotFound("test".to_string());
        assert_eq!(err.status_code(), Status::NotFound);
        
        let err = AppError::ValidationError("test".to_string());
        assert_eq!(err.status_code(), Status::BadRequest);
        
        let err = AppError::RateLimitExceeded(60);
        assert_eq!(err.status_code(), Status::TooManyRequests);
    }

    #[test]
    fn test_error_response() {
        let err = AppError::RateLimitExceeded(60);
        let response = err.to_response();
        
        assert_eq!(response.error_code, "RATE_LIMIT_EXCEEDED");
        assert!(response.details.is_some());
        assert_eq!(response.details.unwrap().retry_after, Some(60));
    }

    #[test]
    fn test_validation_error_helper() {
        let err = validation_error("email", "invalid format");
        assert_eq!(err.to_string(), "Validation error: Field 'email': invalid format");
    }
}
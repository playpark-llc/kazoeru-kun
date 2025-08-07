pub mod session;
pub mod session_counter;
pub mod status_history;
pub mod transaction;

pub use session::{RedisSessionRepository, SessionRepository};
pub use session_counter::{CreateSessionCounter, DbSessionCounterRepository, SessionCounterRepository};
pub use status_history::{CreateStatusHistory, DbStatusHistoryRepository, StatusHistoryRepository};
pub use transaction::{TransactionManager, TransactionManagerImpl};

use crate::config::ConfigError;
use std::fmt;

#[derive(Debug)]
pub enum RepositoryError {
    DatabaseError(String),
    RedisError(String),
    NotFound,
    ConfigError(ConfigError),
    TransactionError(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            RepositoryError::RedisError(msg) => write!(f, "Redis error: {}", msg),
            RepositoryError::NotFound => write!(f, "Resource not found"),
            RepositoryError::ConfigError(err) => write!(f, "Configuration error: {}", err),
            RepositoryError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<sea_orm::DbErr> for RepositoryError {
    fn from(err: sea_orm::DbErr) -> Self {
        RepositoryError::DatabaseError(err.to_string())
    }
}

impl From<deadpool_redis::redis::RedisError> for RepositoryError {
    fn from(err: deadpool_redis::redis::RedisError) -> Self {
        RepositoryError::RedisError(err.to_string())
    }
}

impl From<deadpool_redis::PoolError> for RepositoryError {
    fn from(err: deadpool_redis::PoolError) -> Self {
        RepositoryError::RedisError(err.to_string())
    }
}

impl From<ConfigError> for RepositoryError {
    fn from(err: ConfigError) -> Self {
        RepositoryError::ConfigError(err)
    }
}

pub type Result<T> = std::result::Result<T, RepositoryError>;
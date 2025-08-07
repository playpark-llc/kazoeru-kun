use rocket_cors::AllowedOrigins;
use std::env;

/// Application configuration structure
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub cors: CorsConfig,
    pub target_value: i64,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

/// Redis configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(AppConfig {
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            cors: CorsConfig::from_env()?,
            target_value: Self::load_target_value()?,
        })
    }

    /// Load target value with default
    fn load_target_value() -> Result<i64, ConfigError> {
        match env::var("TARGET_VALUE") {
            Ok(val) => val.parse::<i64>().map_err(|_| {
                ConfigError::InvalidValue(format!("TARGET_VALUE must be a valid integer: {}", val))
            }),
            Err(_) => Ok(100000), // Default value
        }
    }

    /// Get CORS options for Rocket
    pub fn get_cors_options(&self) -> Result<rocket_cors::Cors, rocket_cors::Error> {
        let exact = &self.cors.allowed_origins;
        rocket_cors::CorsOptions::default()
            .allowed_origins(AllowedOrigins::some_exact(exact))
            .to_cors()
    }
}

impl DatabaseConfig {
    /// Load database configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        // Validate URL format (basic check)
        if !url.starts_with("mysql://") && !url.starts_with("postgresql://") {
            return Err(ConfigError::InvalidValue(
                "DATABASE_URL must start with mysql:// or postgresql://".to_string(),
            ));
        }

        Ok(DatabaseConfig { url })
    }
}

impl RedisConfig {
    /// Load Redis configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::MissingEnvVar("REDIS_URL".to_string()))?;

        // Validate URL format (basic check)
        if !url.starts_with("redis://") && !url.contains(":") {
            return Err(ConfigError::InvalidValue(
                "REDIS_URL must be a valid Redis URL or host:port format".to_string(),
            ));
        }

        Ok(RedisConfig { url })
    }
}

impl CorsConfig {
    /// Load CORS configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        let origins_str = env::var("ALLOWED_ORIGINS")
            .map_err(|_| ConfigError::MissingEnvVar("ALLOWED_ORIGINS".to_string()))?;

        let allowed_origins: Vec<String> = origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if allowed_origins.is_empty() {
            return Err(ConfigError::InvalidValue(
                "ALLOWED_ORIGINS must contain at least one origin".to_string(),
            ));
        }

        // Validate each origin
        for origin in &allowed_origins {
            if !origin.starts_with("http://") && !origin.starts_with("https://") {
                return Err(ConfigError::InvalidValue(format!(
                    "Invalid origin format: {}. Origins must start with http:// or https://",
                    origin
                )));
            }
        }

        Ok(CorsConfig { allowed_origins })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to ensure tests don't run in parallel and interfere with env vars
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() {
        env::set_var("DATABASE_URL", "mysql://user:pass@localhost/db");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var(
            "ALLOWED_ORIGINS",
            "http://localhost:3000,https://example.com",
        );
        env::set_var("TARGET_VALUE", "500000");
    }

    fn cleanup_test_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("REDIS_URL");
        env::remove_var("ALLOWED_ORIGINS");
        env::remove_var("TARGET_VALUE");
    }

    #[test]
    fn test_app_config_from_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        setup_test_env();

        let config = AppConfig::from_env().expect("Failed to load config");

        assert_eq!(config.database.url, "mysql://user:pass@localhost/db");
        assert_eq!(config.redis.url, "redis://localhost:6379");
        assert_eq!(config.target_value, 500000);
        assert_eq!(config.cors.allowed_origins.len(), 2);

        cleanup_test_env();
    }

    #[test]
    fn test_database_config_validation() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "invalid_url");
        let result = DatabaseConfig::from_env();
        assert!(result.is_err());

        env::set_var("DATABASE_URL", "mysql://valid_url");
        let result = DatabaseConfig::from_env();
        assert!(result.is_ok());

        cleanup_test_env();
    }

    #[test]
    fn test_redis_config_validation() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("REDIS_URL", "invalid_url");
        let result = RedisConfig::from_env();
        assert!(result.is_err());

        env::set_var("REDIS_URL", "redis://localhost:6379");
        let result = RedisConfig::from_env();
        assert!(result.is_ok());

        cleanup_test_env();
    }

    #[test]
    fn test_cors_config_validation() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("ALLOWED_ORIGINS", "");
        let result = CorsConfig::from_env();
        assert!(result.is_err());

        env::set_var("ALLOWED_ORIGINS", "invalid_origin");
        let result = CorsConfig::from_env();
        assert!(result.is_err());

        env::set_var("ALLOWED_ORIGINS", "http://localhost:3000");
        let result = CorsConfig::from_env();
        assert!(result.is_ok());

        cleanup_test_env();
    }

    #[test]
    fn test_target_value_default() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::remove_var("TARGET_VALUE");
        let config = AppConfig::load_target_value().expect("Failed to load target value");
        assert_eq!(config, 100000); // Should use default

        env::set_var("TARGET_VALUE", "not_a_number");
        let result = AppConfig::load_target_value();
        assert!(result.is_err());

        cleanup_test_env();
    }

    #[test]
    fn test_missing_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();
        cleanup_test_env();

        let result = AppConfig::from_env();
        assert!(result.is_err());

        if let Err(ConfigError::MissingEnvVar(var)) = DatabaseConfig::from_env() {
            assert_eq!(var, "DATABASE_URL");
        } else {
            panic!("Expected MissingEnvVar error");
        }
    }
}

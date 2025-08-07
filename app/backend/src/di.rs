use std::sync::Arc;
use sea_orm::DatabaseConnection;
use deadpool_redis::Pool;
use rocket_cors::Cors;

use crate::config::AppConfig;
use crate::repositories::{
    RedisSessionRepository, SessionRepository,
    DbSessionCounterRepository, SessionCounterRepository,
    DbStatusHistoryRepository, StatusHistoryRepository,
};
use crate::services::{
    SessionService, StatusService, SyncService,
};

/// 依存性注入コンテナ
pub struct DiContainer {
    // Configuration
    pub config: AppConfig,
    
    // Infrastructure
    pub db: Arc<DatabaseConnection>,
    pub redis_pool: Arc<Pool>,
    
    // Repositories
    pub session_repo: Arc<dyn SessionRepository>,
    pub counter_repo: Arc<dyn SessionCounterRepository>,
    pub history_repo: Arc<dyn StatusHistoryRepository>,
    
    // Services
    pub session_service: Arc<SessionService>,
    pub status_service: Arc<StatusService>,
    pub sync_service: Arc<SyncService>,
}

impl DiContainer {
    /// 依存性注入コンテナを構築
    pub async fn build(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Infrastructure層の構築
        let db = Arc::new(create_database_connection(&config.database.url).await?);
        let redis_pool = Arc::new(create_redis_pool(&config.redis)?);
        
        // Repository層の構築
        let session_repo: Arc<dyn SessionRepository> = 
            Arc::new(RedisSessionRepository::new(redis_pool.clone()));
        let counter_repo: Arc<dyn SessionCounterRepository> = 
            Arc::new(DbSessionCounterRepository::new(db.clone()));
        let history_repo: Arc<dyn StatusHistoryRepository> = 
            Arc::new(DbStatusHistoryRepository::new(db.clone()));
        
        // Service層の構築
        let session_service = Arc::new(SessionService::new(session_repo.clone()));
        let sync_service = Arc::new(SyncService::new(
            session_repo.clone(),
            counter_repo.clone(),
            db.clone(),
        ));
        let status_service = Arc::new(StatusService::new(
            history_repo.clone(),
            counter_repo.clone(),
        ));
        
        Ok(Self {
            config,
            db,
            redis_pool,
            session_repo,
            counter_repo,
            history_repo,
            session_service,
            status_service,
            sync_service,
        })
    }
    
    /// CORS設定を取得
    pub fn get_cors(&self) -> Result<Cors, Box<dyn std::error::Error>> {
        self.config.get_cors_options().map_err(|e| e.into())
    }
}

/// データベース接続を作成
async fn create_database_connection(url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    log::info!("Connecting to database...");
    sea_orm::Database::connect(url).await
}

/// Redis接続プールを作成
fn create_redis_pool(config: &crate::config::RedisConfig) -> Result<Pool, Box<dyn std::error::Error>> {
    log::info!("Creating Redis connection pool...");
    
    let mut cfg = deadpool_redis::Config::default();
    cfg.connection = None;
    cfg.url = Some(config.url.clone());
    
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| format!("Failed to create Redis pool: {}", e).into())
}
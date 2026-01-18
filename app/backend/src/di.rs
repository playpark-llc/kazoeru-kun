use deadpool_redis::Pool;
use rocket_cors::Cors;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::repositories::{
    DbSessionCounterRepository, DbStatusHistoryRepository, RedisSessionRepository,
    SessionCounterRepository, SessionRepository, StatusHistoryRepository,
};
use crate::services::{HealthService, SessionService, StatusService, SyncService};

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
    pub health_service: Arc<HealthService>,
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
        let health_service = Arc::new(HealthService::new(db.clone(), redis_pool.clone()));

        // status_historyテーブルの初期化チェック
        Self::ensure_status_history_initialized(&status_service).await?;

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
            health_service,
        })
    }

    /// status_historyテーブルが空の場合、デフォルトレコードで初期化
    async fn ensure_status_history_initialized(
        status_service: &Arc<StatusService>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if status_service.needs_initialization().await? {
            status_service.initialize_with_default().await?;
            log::info!("status_historyテーブルをデフォルトレコードで初期化しました");
        } else {
            log::debug!("status_historyテーブルは既に初期化されています");
        }
        Ok(())
    }

    /// CORS設定を取得
    pub fn get_cors(&self) -> Result<Cors, Box<dyn std::error::Error>> {
        self.config.get_cors_options().map_err(|e| e.into())
    }
}

/// データベース接続を作成
async fn create_database_connection(url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    use sea_orm::ConnectOptions;
    use std::time::Duration;

    log::info!("Connecting to database...");

    // PgBouncer transaction poolingとの互換性のため、prepared statementを無効化
    // 接続URLにstatement-cache-capacity=0を追加
    let url_with_options = if url.contains('?') {
        format!("{}&statement-cache-capacity=0", url)
    } else {
        format!("{}?statement-cache-capacity=0", url)
    };

    let mut opt = ConnectOptions::new(url_with_options);
    opt
        // コネクションプール設定
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        // SQLxログを無効化
        .sqlx_logging(false);

    sea_orm::Database::connect(opt).await
}

/// Redis接続プールを作成
fn create_redis_pool(
    config: &crate::config::RedisConfig,
) -> Result<Pool, Box<dyn std::error::Error>> {
    log::info!("Creating Redis connection pool...");

    let cfg = deadpool_redis::Config {
        connection: None,
        url: Some(config.url.clone()),
        ..Default::default()
    };

    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| format!("Failed to create Redis pool: {e}").into())
}

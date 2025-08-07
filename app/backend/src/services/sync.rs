use std::sync::Arc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};

use crate::repositories::{
    SessionRepository, SessionCounterRepository,
    CreateSessionCounter, RepositoryError
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub total_sessions: usize,
    pub message: String,
    pub success: bool,
}

pub struct SyncService {
    session_repo: Arc<dyn SessionRepository>,
    counter_repo: Arc<dyn SessionCounterRepository>,
    db: Arc<DatabaseConnection>,
}

impl SyncService {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        counter_repo: Arc<dyn SessionCounterRepository>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            session_repo,
            counter_repo,
            db,
        }
    }

    /// RedisのセッションデータをMySQLに同期
    pub async fn sync_sessions_to_database(&self) -> Result<SyncResult, RepositoryError> {
        // Redisから全セッション情報を取得
        let sessions = self.session_repo.get_all_sessions().await?;
        
        if sessions.is_empty() {
            log::info!("No sessions to sync from Redis to MySQL");
            return Ok(SyncResult {
                synced_count: 0,
                total_sessions: 0,
                message: "更新するDBのセッション情報はありませんでした".to_string(),
                success: true,
            });
        }

        let total_sessions = sessions.len();
        log::info!("Syncing {} sessions from Redis to MySQL", total_sessions);
        
        // セッションデータをCreateSessionCounterに変換
        let insert_records: Vec<CreateSessionCounter> = sessions
            .into_iter()
            .map(|session| CreateSessionCounter {
                session_id: session.session_id,
                count: session.count,
            })
            .collect();
        
        // トランザクション内でデータベースにupsert
        let result = self.sync_with_transaction(insert_records).await?;
        
        let message = if result {
            format!("{}件のセッション情報をDBに同期しました", total_sessions)
        } else {
            "セッション同期中にエラーが発生しました".to_string()
        };
        
        Ok(SyncResult {
            synced_count: if result { total_sessions } else { 0 },
            total_sessions,
            message,
            success: result,
        })
    }
    
    /// トランザクション内で同期を実行
    async fn sync_with_transaction(&self, records: Vec<CreateSessionCounter>) -> Result<bool, RepositoryError> {
        let txn = self.db
            .begin()
            .await
            .map_err(|e| RepositoryError::TransactionError(format!("Failed to begin transaction: {:?}", e)))?;
        
        match self.counter_repo.upsert_many_with_txn(records, &txn).await {
            Ok(_) => {
                txn.commit()
                    .await
                    .map_err(|e| RepositoryError::TransactionError(format!("Failed to commit transaction: {:?}", e)))?;
                Ok(true)
            }
            Err(e) => {
                let _ = txn.rollback().await;
                log::error!("Failed to upsert session_counter table: {:?}", e);
                Err(e)
            }
        }
    }
    
    /// 期限切れセッションのクリーンアップ
    pub async fn cleanup_expired_sessions(&self) -> Result<usize, RepositoryError> {
        let sessions = self.session_repo.get_all_sessions().await?;
        let mut cleaned_count = 0;
        
        for session in sessions {
            // 有効期限が設定されていて、期限切れのセッションを削除
            if let Some(expiration) = session.expiration {
                let now = chrono::Local::now().timestamp();
                if expiration < now {
                    log::info!("Cleaning up expired session: {}", session.session_id);
                    self.session_repo.delete(&session.session_id).await?;
                    cleaned_count += 1;
                }
            }
        }
        
        if cleaned_count > 0 {
            log::info!("Cleaned up {} expired sessions", cleaned_count);
        }
        
        Ok(cleaned_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // テスト用のモックリポジトリ
    struct MockSessionRepository {
        sessions: Arc<Mutex<HashMap<String, (i32, Option<i64>)>>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        fn add_session(&self, session_id: &str, count: i32, expiration: Option<i64>) {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id.to_string(), (count, expiration));
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn increment(&self, _session_id: &str) -> Result<u32, RepositoryError> {
            unimplemented!()
        }

        async fn set_expiration(&self, _session_id: &str, _expiration: i64) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn get_count(&self, _session_id: &str) -> Result<i32, RepositoryError> {
            unimplemented!()
        }

        async fn get_all_sessions(&self) -> Result<Vec<crate::repositories::session::SessionData>, RepositoryError> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions
                .iter()
                .map(|(id, (count, expiration))| crate::repositories::session::SessionData {
                    session_id: id.clone(),
                    count: *count,
                    expiration: *expiration,
                })
                .collect())
        }

        async fn delete(&self, session_id: &str) -> Result<(), RepositoryError> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id);
            Ok(())
        }

        async fn exists(&self, _session_id: &str) -> Result<bool, RepositoryError> {
            unimplemented!()
        }
    }

    struct MockSessionCounterRepository {
        upserted_records: Arc<Mutex<Vec<CreateSessionCounter>>>,
    }

    impl MockSessionCounterRepository {
        fn new() -> Self {
            Self {
                upserted_records: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl SessionCounterRepository for MockSessionCounterRepository {
        async fn upsert(&self, _data: CreateSessionCounter) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn upsert_many(&self, _data: Vec<CreateSessionCounter>) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn upsert_many_with_txn(&self, data: Vec<CreateSessionCounter>, _txn: &sea_orm::DatabaseTransaction) -> Result<(), RepositoryError> {
            let mut records = self.upserted_records.lock().unwrap();
            records.extend(data);
            Ok(())
        }

        async fn sum_count_between(&self, _start_date: chrono::DateTime<chrono::Local>, _end_date: chrono::DateTime<chrono::Local>) -> Result<i64, RepositoryError> {
            unimplemented!()
        }

        async fn find_by_session_id(&self, _session_id: &str) -> Result<Option<crate::repositories::session_counter::SessionCounterData>, RepositoryError> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: i32) -> Result<Option<crate::repositories::session_counter::SessionCounterData>, RepositoryError> {
            unimplemented!()
        }

        async fn list(&self, _limit: u64, _offset: u64) -> Result<(Vec<crate::repositories::session_counter::SessionCounterData>, u64), RepositoryError> {
            unimplemented!()
        }

        async fn list_by_date_range(&self, _start_date: chrono::DateTime<chrono::Local>, _end_date: chrono::DateTime<chrono::Local>, _limit: u64, _offset: u64) -> Result<(Vec<crate::repositories::session_counter::SessionCounterData>, u64), RepositoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: i32) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn delete_by_session_id(&self, _session_id: &str) -> Result<(), RepositoryError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_sync_empty_sessions() {
        let session_repo = Arc::new(MockSessionRepository::new());
        let counter_repo = Arc::new(MockSessionCounterRepository::new());
        
        // テスト用のインメモリDBを作成（実際のDBは使用しない）
        let db = Arc::new(sea_orm::Database::connect("sqlite::memory:").await.unwrap());
        
        let service = SyncService::new(session_repo, counter_repo, db);

        let result = service.sync_sessions_to_database().await.unwrap();
        
        assert_eq!(result.synced_count, 0);
        assert_eq!(result.total_sessions, 0);
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let session_repo = Arc::new(MockSessionRepository::new());
        let counter_repo = Arc::new(MockSessionCounterRepository::new());
        let db = Arc::new(sea_orm::Database::connect("sqlite::memory:").await.unwrap());
        
        // 期限切れと有効なセッションを追加
        let now = chrono::Local::now().timestamp();
        session_repo.add_session("expired1", 5, Some(now - 3600)); // 1時間前に期限切れ
        session_repo.add_session("expired2", 3, Some(now - 100)); // 100秒前に期限切れ
        session_repo.add_session("valid1", 10, Some(now + 3600)); // まだ有効
        session_repo.add_session("valid2", 8, None); // 期限なし
        
        let service = SyncService::new(session_repo.clone(), counter_repo, db);

        let cleaned_count = service.cleanup_expired_sessions().await.unwrap();
        
        assert_eq!(cleaned_count, 2); // 2つの期限切れセッションが削除される
        
        // 残りのセッションを確認
        let remaining_sessions = session_repo.get_all_sessions().await.unwrap();
        assert_eq!(remaining_sessions.len(), 2);
    }
}
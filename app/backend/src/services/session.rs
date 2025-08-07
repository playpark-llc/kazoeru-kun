use std::sync::Arc;
use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::repositories::{SessionRepository, RepositoryError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub count: u32,
    pub message: String,
    pub is_new: bool,
}

pub struct SessionService {
    session_repo: Arc<dyn SessionRepository>,
}

impl SessionService {
    pub fn new(session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { session_repo }
    }

    /// セッションをインクリメントし、必要に応じて新規作成・有効期限設定を行う
    pub async fn increment_session(&self, session_id: Option<&str>) -> Result<SessionResponse, RepositoryError> {
        let session_id = match session_id {
            Some(id) if !id.is_empty() => {
                // 既存セッションIDがある場合、有効性をチェック
                let exists = self.session_repo.exists(id).await?;
                if exists {
                    id.to_string()
                } else {
                    // セッションが存在しない場合は新規作成
                    self.create_new_session().await?
                }
            }
            _ => {
                // セッションIDがない場合は新規作成
                self.create_new_session().await?
            }
        };

        // カウントをインクリメント
        let count = self.session_repo.increment(&session_id).await?;
        
        // 新規セッション（カウント1）の場合は有効期限を設定
        let (message, is_new) = if count == 1 {
            let expiration = self.calculate_expiration();
            self.session_repo
                .set_expiration(&session_id, expiration.timestamp())
                .await?;
            ("新規セッションが登録されました".to_string(), true)
        } else {
            ("セッションカウントが更新されました".to_string(), false)
        };

        Ok(SessionResponse {
            session_id,
            count,
            message,
            is_new,
        })
    }

    /// 新しいセッションIDを生成
    async fn create_new_session(&self) -> Result<String, RepositoryError> {
        let session_id = self.generate_session_id();
        log::info!("Creating new session: {}", session_id);
        Ok(session_id)
    }

    /// ランダムなセッションIDを生成
    fn generate_session_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// セッションの有効期限を計算（翌日の0:00:00）
    fn calculate_expiration(&self) -> chrono::DateTime<Local> {
        use chrono::TimeZone;
        
        Local::now()
            .date_naive()
            .succ_opt()
            .and_then(|tomorrow| tomorrow.and_hms_opt(0, 0, 0))
            .and_then(|dt| Local.from_local_datetime(&dt).single())
            .unwrap_or_else(|| {
                // フォールバック: 現在時刻から24時間後
                Local::now() + chrono::Duration::hours(24)
            })
    }

    /// セッション情報を取得
    pub async fn get_session_count(&self, session_id: &str) -> Result<i32, RepositoryError> {
        self.session_repo.get_count(session_id).await
    }

    /// セッションを削除
    pub async fn delete_session(&self, session_id: &str) -> Result<(), RepositoryError> {
        self.session_repo.delete(session_id).await
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
        sessions: Arc<Mutex<HashMap<String, u32>>>,
        expirations: Arc<Mutex<HashMap<String, i64>>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Arc::new(Mutex::new(HashMap::new())),
                expirations: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn increment(&self, session_id: &str) -> Result<u32, RepositoryError> {
            let mut sessions = self.sessions.lock().unwrap();
            let count = sessions.entry(session_id.to_string()).or_insert(0);
            *count += 1;
            Ok(*count)
        }

        async fn set_expiration(&self, session_id: &str, expiration: i64) -> Result<(), RepositoryError> {
            let mut expirations = self.expirations.lock().unwrap();
            expirations.insert(session_id.to_string(), expiration);
            Ok(())
        }

        async fn get_count(&self, session_id: &str) -> Result<i32, RepositoryError> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.get(session_id).copied().unwrap_or(0) as i32)
        }

        async fn get_all_sessions(&self) -> Result<Vec<crate::repositories::session::SessionData>, RepositoryError> {
            let sessions = self.sessions.lock().unwrap();
            let expirations = self.expirations.lock().unwrap();
            
            Ok(sessions
                .iter()
                .map(|(id, count)| crate::repositories::session::SessionData {
                    session_id: id.clone(),
                    count: *count as i32,
                    expiration: expirations.get(id).copied(),
                })
                .collect())
        }

        async fn delete(&self, session_id: &str) -> Result<(), RepositoryError> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id);
            let mut expirations = self.expirations.lock().unwrap();
            expirations.remove(session_id);
            Ok(())
        }

        async fn exists(&self, session_id: &str) -> Result<bool, RepositoryError> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.contains_key(session_id))
        }
    }

    #[tokio::test]
    async fn test_new_session_creation() {
        let repo = Arc::new(MockSessionRepository::new());
        let service = SessionService::new(repo.clone());

        let response = service.increment_session(None).await.unwrap();
        
        assert_eq!(response.count, 1);
        assert!(response.is_new);
        assert!(!response.session_id.is_empty());
        assert_eq!(response.message, "新規セッションが登録されました");
    }

    #[tokio::test]
    async fn test_existing_session_increment() {
        let repo = Arc::new(MockSessionRepository::new());
        let service = SessionService::new(repo.clone());

        // 最初のインクリメント（新規作成）
        let response1 = service.increment_session(None).await.unwrap();
        let session_id = response1.session_id.clone();
        
        // 2回目のインクリメント（既存セッション）
        let response2 = service.increment_session(Some(&session_id)).await.unwrap();
        
        assert_eq!(response2.session_id, session_id);
        assert_eq!(response2.count, 2);
        assert!(!response2.is_new);
        assert_eq!(response2.message, "セッションカウントが更新されました");
    }

    #[tokio::test]
    async fn test_nonexistent_session_creates_new() {
        let repo = Arc::new(MockSessionRepository::new());
        let service = SessionService::new(repo.clone());

        // 存在しないセッションIDを指定
        let response = service.increment_session(Some("nonexistent")).await.unwrap();
        
        assert_ne!(response.session_id, "nonexistent");
        assert_eq!(response.count, 1);
        assert!(response.is_new);
    }

    #[tokio::test]
    async fn test_get_session_count() {
        let repo = Arc::new(MockSessionRepository::new());
        let service = SessionService::new(repo.clone());

        let response = service.increment_session(None).await.unwrap();
        let session_id = response.session_id.clone();
        
        // カウントを複数回インクリメント
        service.increment_session(Some(&session_id)).await.unwrap();
        service.increment_session(Some(&session_id)).await.unwrap();
        
        let count = service.get_session_count(&session_id).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let repo = Arc::new(MockSessionRepository::new());
        let service = SessionService::new(repo.clone());

        let response = service.increment_session(None).await.unwrap();
        let session_id = response.session_id.clone();
        
        // セッションを削除
        service.delete_session(&session_id).await.unwrap();
        
        // 削除後のカウントは0になる
        let count = service.get_session_count(&session_id).await.unwrap();
        assert_eq!(count, 0);
    }
}
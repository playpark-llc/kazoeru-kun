use async_trait::async_trait;
use deadpool_redis::{
    redis::{self, AsyncCommands},
    Pool,
};
use std::sync::Arc;

use super::Result;

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: String,
    pub count: i32,
    pub expiration: Option<i64>,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn increment(&self, session_id: &str) -> Result<u32>;
    async fn set_expiration(&self, session_id: &str, expiration: i64) -> Result<()>;
    async fn get_count(&self, session_id: &str) -> Result<i32>;
    async fn get_all_sessions(&self) -> Result<Vec<SessionData>>;
    async fn delete(&self, session_id: &str) -> Result<()>;
    async fn exists(&self, session_id: &str) -> Result<bool>;
}

pub struct RedisSessionRepository {
    pool: Arc<Pool>,
}

impl RedisSessionRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for RedisSessionRepository {
    async fn increment(&self, session_id: &str) -> Result<u32> {
        let mut conn = self.pool.get().await?;
        let count: u32 = conn.incr(session_id, 1).await?;
        Ok(count)
    }

    async fn set_expiration(&self, session_id: &str, expiration: i64) -> Result<()> {
        let mut conn = self.pool.get().await?;
        conn.expire_at::<&str, u32>(session_id, expiration).await?;
        Ok(())
    }

    async fn get_count(&self, session_id: &str) -> Result<i32> {
        let mut conn = self.pool.get().await?;
        let count: i32 = conn.get(session_id).await.unwrap_or(0);
        Ok(count)
    }

    async fn get_all_sessions(&self) -> Result<Vec<SessionData>> {
        let mut conn = self.pool.get().await?;
        
        // Use SCAN iterator instead of KEYS * to avoid blocking Redis
        let keys: Vec<String> = {
            let mut keys = Vec::new();
            let mut iter = conn.scan_match::<_, String>("*").await?;
            while let Some(key) = iter.next_item().await {
                keys.push(key);
            }
            keys
        };
        
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        
        // Use pipeline for batch operations
        let mut pipe = redis::pipe();
        for key in &keys {
            pipe.get(key);
            pipe.ttl(key);
        }
        
        let results: Vec<redis::Value> = pipe.query_async(&mut conn).await?;
        
        // Process results in pairs (get result, ttl result)
        for (i, key) in keys.iter().enumerate() {
            let count_idx = i * 2;
            let ttl_idx = i * 2 + 1;
            
            if let Some(redis::Value::Data(data)) = results.get(count_idx) {
                let count: i32 = String::from_utf8_lossy(data).parse().unwrap_or(0);
                
                let ttl = if let Some(redis::Value::Int(ttl_val)) = results.get(ttl_idx) {
                    *ttl_val
                } else {
                    -1
                };
                
                let expiration = if ttl > 0 {
                    Some(chrono::Local::now().timestamp() + ttl)
                } else {
                    None
                };
                
                sessions.push(SessionData {
                    session_id: key.clone(),
                    count,
                    expiration,
                });
            }
        }

        Ok(sessions)
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(session_id).await?;
        Ok(())
    }

    async fn exists(&self, session_id: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let exists: bool = conn.exists(session_id).await?;
        Ok(exists)
    }
}

#[cfg(test)]
#[cfg(feature = "integration_tests")]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};
    
    async fn create_test_pool() -> Pool {
        let cfg = Config::from_url("redis://localhost:6379");
        cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
    }
    
    #[tokio::test]
    async fn test_increment() {
        let pool = create_test_pool().await;
        let repo = RedisSessionRepository::new(Arc::new(pool.clone()));
        
        let session_id = "test_session_increment";
        
        let mut conn = pool.get().await.unwrap();
        let _: () = conn.del(session_id).await.unwrap();
        
        let count = repo.increment(session_id).await.unwrap();
        assert_eq!(count, 1);
        
        let count = repo.increment(session_id).await.unwrap();
        assert_eq!(count, 2);
        
        let _: () = conn.del(session_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_set_expiration() {
        let pool = create_test_pool().await;
        let repo = RedisSessionRepository::new(Arc::new(pool.clone()));
        
        let session_id = "test_session_expiration";
        let expiration = chrono::Local::now().timestamp() + 3600;
        
        repo.increment(session_id).await.unwrap();
        repo.set_expiration(session_id, expiration).await.unwrap();
        
        let mut conn = pool.get().await.unwrap();
        let ttl: i64 = conn.ttl(session_id).await.unwrap();
        assert!(ttl > 0 && ttl <= 3600);
        
        let _: () = conn.del(session_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_get_count() {
        let pool = create_test_pool().await;
        let repo = RedisSessionRepository::new(Arc::new(pool.clone()));
        
        let session_id = "test_session_get_count";
        
        let count = repo.get_count(session_id).await.unwrap();
        assert_eq!(count, 0);
        
        repo.increment(session_id).await.unwrap();
        repo.increment(session_id).await.unwrap();
        
        let count = repo.get_count(session_id).await.unwrap();
        assert_eq!(count, 2);
        
        let mut conn = pool.get().await.unwrap();
        let _: () = conn.del(session_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_exists() {
        let pool = create_test_pool().await;
        let repo = RedisSessionRepository::new(Arc::new(pool.clone()));
        
        let session_id = "test_session_exists";
        
        let exists = repo.exists(session_id).await.unwrap();
        assert!(!exists);
        
        repo.increment(session_id).await.unwrap();
        
        let exists = repo.exists(session_id).await.unwrap();
        assert!(exists);
        
        repo.delete(session_id).await.unwrap();
        
        let exists = repo.exists(session_id).await.unwrap();
        assert!(!exists);
    }
}
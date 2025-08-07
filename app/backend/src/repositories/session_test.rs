#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::session::{RedisSessionRepository, SessionRepository};
    use deadpool_redis::{Config, Pool, Runtime};
    use std::sync::Arc;

    async fn create_test_pool() -> Pool {
        let cfg = Config::from_url("redis://redis:6379");
        cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
    }

    async fn create_test_repo() -> RedisSessionRepository {
        let pool = create_test_pool().await;
        RedisSessionRepository::new(Arc::new(pool))
    }

    #[tokio::test]
    async fn test_session_validation() {
        let repo = create_test_repo().await;
        let session_id = "test_session_validation";
        
        // Session should not be valid initially
        let is_valid = repo.is_valid(session_id).await.unwrap();
        assert!(!is_valid);
        
        // Create session
        let count = repo.increment(session_id).await.unwrap();
        assert_eq!(count, 1);
        
        // Session should be valid after creation
        let is_valid = repo.is_valid(session_id).await.unwrap();
        assert!(is_valid);
        
        // Set expiration
        let expiration = chrono::Local::now().timestamp() + 3600;
        repo.set_expiration(session_id, expiration).await.unwrap();
        
        // Session should still be valid
        let is_valid = repo.is_valid(session_id).await.unwrap();
        assert!(is_valid);
        
        // Check TTL
        let ttl = repo.get_ttl(session_id).await.unwrap();
        assert!(ttl > 0 && ttl <= 3600);
        
        // Clean up
        repo.delete(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_expired_session_handling() {
        let repo = create_test_repo().await;
        let session_id = "test_expired_session";
        
        // Create session with very short expiration
        repo.increment(session_id).await.unwrap();
        let expiration = chrono::Local::now().timestamp() + 1; // 1 second
        repo.set_expiration(session_id, expiration).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Session should not be valid after expiration
        let is_valid = repo.is_valid(session_id).await.unwrap();
        assert!(!is_valid);
        
        // TTL should be -2 (key doesn't exist)
        let ttl = repo.get_ttl(session_id).await.unwrap();
        assert_eq!(ttl, -2);
    }

    #[tokio::test]
    async fn test_get_count_with_expired_session() {
        let repo = create_test_repo().await;
        let session_id = "test_get_count_expired";
        
        // Create session
        repo.increment(session_id).await.unwrap();
        repo.increment(session_id).await.unwrap();
        
        // Count should be 2
        let count = repo.get_count(session_id).await.unwrap();
        assert_eq!(count, 2);
        
        // Set very short expiration
        let expiration = chrono::Local::now().timestamp() + 1;
        repo.set_expiration(session_id, expiration).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Count should be 0 for expired session
        let count = repo.get_count(session_id).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_all_sessions_with_expiration() {
        let repo = create_test_repo().await;
        let session_id1 = "test_all_sessions_1";
        let session_id2 = "test_all_sessions_2";
        
        // Create multiple sessions with different expiration times
        repo.increment(session_id1).await.unwrap();
        repo.increment(session_id1).await.unwrap();
        
        repo.increment(session_id2).await.unwrap();
        repo.increment(session_id2).await.unwrap();
        repo.increment(session_id2).await.unwrap();
        
        // Set expirations
        let expiration1 = chrono::Local::now().timestamp() + 3600;
        let expiration2 = chrono::Local::now().timestamp() + 7200;
        
        repo.set_expiration(session_id1, expiration1).await.unwrap();
        repo.set_expiration(session_id2, expiration2).await.unwrap();
        
        // Get all sessions
        let sessions = repo.get_all_sessions().await.unwrap();
        
        // Find our test sessions
        let session1 = sessions.iter().find(|s| s.session_id == session_id1);
        let session2 = sessions.iter().find(|s| s.session_id == session_id2);
        
        assert!(session1.is_some());
        assert!(session2.is_some());
        
        let session1 = session1.unwrap();
        let session2 = session2.unwrap();
        
        assert_eq!(session1.count, 2);
        assert_eq!(session2.count, 3);
        
        assert!(session1.expiration.is_some());
        assert!(session2.expiration.is_some());
        
        // Clean up
        repo.delete(session_id1).await.unwrap();
        repo.delete(session_id2).await.unwrap();
    }
}
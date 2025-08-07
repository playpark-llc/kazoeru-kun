use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::sync::Arc;

use super::{RepositoryError, Result};

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<DatabaseTransaction>;
    async fn commit(&self, txn: DatabaseTransaction) -> Result<()>;
    async fn rollback(&self, txn: DatabaseTransaction) -> Result<()>;
    async fn with_transaction<F, R, Fut>(&self, f: F) -> Result<R>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send;
}

pub struct TransactionManagerImpl {
    db: Arc<DatabaseConnection>,
}

impl TransactionManagerImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TransactionManager for TransactionManagerImpl {
    async fn begin(&self) -> Result<DatabaseTransaction> {
        self.db
            .begin()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))
    }

    async fn commit(&self, txn: DatabaseTransaction) -> Result<()> {
        txn.commit()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))
    }

    async fn rollback(&self, txn: DatabaseTransaction) -> Result<()> {
        txn.rollback()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))
    }

    async fn with_transaction<F, R, Fut>(&self, f: F) -> Result<R>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send,
    {
        let txn = self.begin().await?;
        
        match f(txn).await {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "integration_tests")]
mod tests {
    use super::*;
    use crate::sea_generated::{prelude::SessionCounter, session_counter};
    use sea_orm::{Database, EntityTrait, Set};

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("mysql://user:password@localhost/test_db")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let db = Arc::new(create_test_db().await);
        let tx_manager = TransactionManagerImpl::new(db.clone());

        let result = tx_manager
            .with_transaction(|txn| async move {
                let active_model = session_counter::ActiveModel {
                    session_id: Set("tx_test_commit".to_string()),
                    count: Set(10),
                    ..Default::default()
                };

                SessionCounter::insert(active_model)
                    .exec(&txn)
                    .await
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                txn.commit()
                    .await
                    .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;

                Ok(())
            })
            .await;

        assert!(result.is_ok());

        let found = SessionCounter::find()
            .filter(session_counter::Column::SessionId.eq("tx_test_commit"))
            .one(&*db)
            .await
            .unwrap();

        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let db = Arc::new(create_test_db().await);
        let tx_manager = TransactionManagerImpl::new(db.clone());

        let result = tx_manager
            .with_transaction(|txn| async move {
                let active_model = session_counter::ActiveModel {
                    session_id: Set("tx_test_rollback".to_string()),
                    count: Set(20),
                    ..Default::default()
                };

                SessionCounter::insert(active_model)
                    .exec(&txn)
                    .await
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                txn.rollback()
                    .await
                    .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;

                Ok(())
            })
            .await;

        assert!(result.is_ok());

        let found = SessionCounter::find()
            .filter(session_counter::Column::SessionId.eq("tx_test_rollback"))
            .one(&*db)
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_transaction_with_error() {
        let db = Arc::new(create_test_db().await);
        let tx_manager = TransactionManagerImpl::new(db.clone());

        let result: Result<()> = tx_manager
            .with_transaction(|txn| async move {
                let active_model = session_counter::ActiveModel {
                    session_id: Set("tx_test_error".to_string()),
                    count: Set(30),
                    ..Default::default()
                };

                SessionCounter::insert(active_model)
                    .exec(&txn)
                    .await
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                Err(RepositoryError::DatabaseError(
                    "Simulated error".to_string(),
                ))
            })
            .await;

        assert!(result.is_err());

        let found = SessionCounter::find()
            .filter(session_counter::Column::SessionId.eq("tx_test_error"))
            .one(&*db)
            .await
            .unwrap();

        assert!(found.is_none());
    }
}
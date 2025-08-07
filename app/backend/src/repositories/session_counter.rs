use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::{
    models::response::SessionCounterSummary,
    sea_generated::{prelude::SessionCounter, session_counter},
};

use super::{RepositoryError, Result};

#[derive(Debug, Clone)]
pub struct SessionCounterData {
    pub id: i32,
    pub session_id: String,
    pub count: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionCounter {
    pub session_id: String,
    pub count: i32,
}

#[async_trait]
pub trait SessionCounterRepository: Send + Sync {
    async fn upsert(&self, data: CreateSessionCounter) -> Result<()>;
    async fn upsert_many(&self, data: Vec<CreateSessionCounter>) -> Result<()>;
    async fn upsert_many_with_txn(
        &self,
        data: Vec<CreateSessionCounter>,
        txn: &DatabaseTransaction,
    ) -> Result<()>;
    async fn find_by_session_id(&self, session_id: &str) -> Result<Option<SessionCounterData>>;
    async fn find_by_id(&self, id: i32) -> Result<Option<SessionCounterData>>;
    async fn list(&self, limit: u64, offset: u64) -> Result<(Vec<SessionCounterData>, u64)>;
    async fn sum_count_between(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
    ) -> Result<i64>;
    async fn delete(&self, id: i32) -> Result<()>;
    async fn delete_by_session_id(&self, session_id: &str) -> Result<()>;
}

pub struct DbSessionCounterRepository {
    db: Arc<DatabaseConnection>,
}

impl DbSessionCounterRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_data(entity: session_counter::Model) -> SessionCounterData {
        SessionCounterData {
            id: entity.id,
            session_id: entity.session_id,
            count: entity.count,
            created_at: Local.from_utc_datetime(&entity.created_at.naive_utc()),
            updated_at: Local.from_utc_datetime(&entity.updated_at.naive_utc()),
        }
    }
}

#[async_trait]
impl SessionCounterRepository for DbSessionCounterRepository {
    async fn upsert(&self, data: CreateSessionCounter) -> Result<()> {
        let active_model = session_counter::ActiveModel {
            session_id: Set(data.session_id.clone()),
            count: Set(data.count),
            ..Default::default()
        };

        let on_conflict = OnConflict::column(session_counter::Column::SessionId)
            .update_column(session_counter::Column::Count)
            .to_owned();

        let result = SessionCounter::insert(active_model)
            .on_conflict(on_conflict)
            .exec(&*self.db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sea_orm::DbErr::RecordNotInserted) => {
                // This is expected for upsert when record exists and is updated
                Ok(())
            }
            Err(e) => Err(RepositoryError::from(e)),
        }
    }

    async fn upsert_many(&self, data: Vec<CreateSessionCounter>) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let active_models: Vec<session_counter::ActiveModel> = data
            .into_iter()
            .map(|d| session_counter::ActiveModel {
                session_id: Set(d.session_id),
                count: Set(d.count),
                ..Default::default()
            })
            .collect();

        let on_conflict = OnConflict::column(session_counter::Column::SessionId)
            .update_column(session_counter::Column::Count)
            .to_owned();

        let result = SessionCounter::insert_many(active_models)
            .on_conflict(on_conflict)
            .exec(&*self.db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sea_orm::DbErr::RecordNotInserted) => {
                // RecordNotInserted is expected behavior for upsert operations
                // when the record already exists and is updated instead of inserted.
                // This is not an error condition for our use case.
                Ok(())
            }
            Err(e) => Err(RepositoryError::from(e)),
        }
    }

    async fn upsert_many_with_txn(
        &self,
        data: Vec<CreateSessionCounter>,
        txn: &DatabaseTransaction,
    ) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let active_models: Vec<session_counter::ActiveModel> = data
            .into_iter()
            .map(|d| session_counter::ActiveModel {
                session_id: Set(d.session_id),
                count: Set(d.count),
                ..Default::default()
            })
            .collect();

        let on_conflict = OnConflict::column(session_counter::Column::SessionId)
            .update_column(session_counter::Column::Count)
            .to_owned();

        let result = SessionCounter::insert_many(active_models)
            .on_conflict(on_conflict)
            .exec(txn)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sea_orm::DbErr::RecordNotInserted) => {
                // RecordNotInserted is expected behavior for upsert operations
                // when the record already exists and is updated instead of inserted.
                // This is not an error condition for our use case.
                Ok(())
            }
            Err(e) => Err(RepositoryError::from(e)),
        }
    }

    async fn find_by_session_id(&self, session_id: &str) -> Result<Option<SessionCounterData>> {
        let entity = SessionCounter::find()
            .filter(session_counter::Column::SessionId.eq(session_id))
            .one(&*self.db)
            .await?;

        Ok(entity.map(Self::entity_to_data))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<SessionCounterData>> {
        let entity = SessionCounter::find_by_id(id).one(&*self.db).await?;

        Ok(entity.map(Self::entity_to_data))
    }

    async fn list(&self, limit: u64, offset: u64) -> Result<(Vec<SessionCounterData>, u64)> {
        let paginator = SessionCounter::find()
            .order_by_desc(session_counter::Column::UpdatedAt)
            .paginate(&*self.db, limit);

        let total = paginator.num_items().await?;
        let entities = paginator.fetch_page(offset / limit).await?;

        let data: Vec<SessionCounterData> =
            entities.into_iter().map(Self::entity_to_data).collect();

        Ok((data, total))
    }

    async fn sum_count_between(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
    ) -> Result<i64> {
        let result = SessionCounter::find()
            .filter(session_counter::Column::UpdatedAt.between(start_date, end_date))
            .select_only()
            .column_as(session_counter::Column::Count.sum(), "sum_count")
            .into_model::<SessionCounterSummary>()
            .all(&*self.db)
            .await?;

        let sum = result
            .first()
            .and_then(|r| r.sum_count.to_string().parse::<i64>().ok())
            .unwrap_or(0);

        Ok(sum)
    }

    async fn delete(&self, id: i32) -> Result<()> {
        SessionCounter::delete_by_id(id).exec(&*self.db).await?;
        Ok(())
    }

    async fn delete_by_session_id(&self, session_id: &str) -> Result<()> {
        SessionCounter::delete_many()
            .filter(session_counter::Column::SessionId.eq(session_id))
            .exec(&*self.db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "integration_tests")]
mod tests {
    use super::*;
    use sea_orm::Database;

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("mysql://user:password@localhost/test_db")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_upsert() {
        let db = create_test_db().await;
        let repo = DbSessionCounterRepository::new(Arc::new(db));

        let data = CreateSessionCounter {
            session_id: "test_session_1".to_string(),
            count: 5,
        };

        let result = repo.upsert(data.clone()).await;
        assert!(result.is_ok());

        let updated_data = CreateSessionCounter {
            session_id: "test_session_1".to_string(),
            count: 10,
        };

        let result = repo.upsert(updated_data).await;
        assert!(result.is_ok());

        let found = repo.find_by_session_id("test_session_1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().count, 10);
    }

    #[tokio::test]
    async fn test_upsert_many() {
        let db = create_test_db().await;
        let repo = DbSessionCounterRepository::new(Arc::new(db));

        let data = vec![
            CreateSessionCounter {
                session_id: "test_session_2".to_string(),
                count: 5,
            },
            CreateSessionCounter {
                session_id: "test_session_3".to_string(),
                count: 10,
            },
            CreateSessionCounter {
                session_id: "test_session_4".to_string(),
                count: 15,
            },
        ];

        let result = repo.upsert_many(data).await;
        assert!(result.is_ok());

        let found = repo.find_by_session_id("test_session_3").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().count, 10);
    }

    #[tokio::test]
    async fn test_sum_count_between() {
        let db = create_test_db().await;
        let repo = DbSessionCounterRepository::new(Arc::new(db));

        let now = Local::now();
        let start_date = now - chrono::Duration::days(7);
        let end_date = now + chrono::Duration::days(1);

        let data = vec![
            CreateSessionCounter {
                session_id: "test_session_5".to_string(),
                count: 10,
            },
            CreateSessionCounter {
                session_id: "test_session_6".to_string(),
                count: 20,
            },
            CreateSessionCounter {
                session_id: "test_session_7".to_string(),
                count: 30,
            },
        ];

        repo.upsert_many(data).await.unwrap();

        let sum = repo.sum_count_between(start_date, end_date).await.unwrap();
        assert!(sum >= 60);
    }
}

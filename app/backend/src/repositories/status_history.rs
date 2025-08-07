use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use sea_orm::{
    prelude::Decimal, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::sea_generated::{
    prelude::StatusHistory, sea_orm_active_enums::Status, status_history,
};

use super::{RepositoryError, Result};

#[derive(Debug, Clone)]
pub struct StatusHistoryData {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub count: Decimal,
    pub status: Status,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct CreateStatusHistory {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub count: Decimal,
    pub status: Status,
}

#[async_trait]
pub trait StatusHistoryRepository: Send + Sync {
    async fn create(&self, data: CreateStatusHistory) -> Result<StatusHistoryData>;
    async fn find_by_id(&self, id: i32) -> Result<Option<StatusHistoryData>>;
    async fn find_by_date_range(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
    ) -> Result<Vec<StatusHistoryData>>;
    async fn get_latest(&self) -> Result<Option<StatusHistoryData>>;
    async fn list(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<StatusHistoryData>, u64)>;
    async fn list_by_date_range(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<StatusHistoryData>, u64)>;
    async fn update_status(&self, id: i32, status: Status) -> Result<()>;
    async fn delete(&self, id: i32) -> Result<()>;
}

pub struct DbStatusHistoryRepository {
    db: Arc<DatabaseConnection>,
}

impl DbStatusHistoryRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_data(entity: status_history::Model) -> StatusHistoryData {
        StatusHistoryData {
            id: entity.id,
            start_date: entity.start_date.date(),
            end_date: entity.end_date.date(),
            count: entity.count,
            status: entity.status,
            created_at: Local.from_utc_datetime(&entity.created_at.naive_utc()),
            updated_at: Local.from_utc_datetime(&entity.updated_at.naive_utc()),
        }
    }
}

#[async_trait]
impl StatusHistoryRepository for DbStatusHistoryRepository {
    async fn create(&self, data: CreateStatusHistory) -> Result<StatusHistoryData> {
        let active_model = status_history::ActiveModel {
            start_date: Set(data.start_date.and_hms_opt(0, 0, 0).unwrap()),
            end_date: Set(data.end_date.and_hms_opt(23, 59, 59).unwrap()),
            count: Set(data.count),
            status: Set(data.status),
            ..Default::default()
        };

        let result = StatusHistory::insert(active_model)
            .exec(&*self.db)
            .await?;

        let entity = StatusHistory::find_by_id(result.last_insert_id)
            .one(&*self.db)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::entity_to_data(entity))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<StatusHistoryData>> {
        let entity = StatusHistory::find_by_id(id).one(&*self.db).await?;

        Ok(entity.map(Self::entity_to_data))
    }

    async fn find_by_date_range(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
    ) -> Result<Vec<StatusHistoryData>> {
        let entities = StatusHistory::find()
            .filter(status_history::Column::StartDate.gte(start_date.naive_local()))
            .filter(status_history::Column::EndDate.lte(end_date.naive_local()))
            .all(&*self.db)
            .await?;

        Ok(entities.into_iter().map(Self::entity_to_data).collect())
    }

    async fn get_latest(&self) -> Result<Option<StatusHistoryData>> {
        let entity = StatusHistory::find()
            .order_by_desc(status_history::Column::CreatedAt)
            .limit(1)
            .one(&*self.db)
            .await?;

        Ok(entity.map(Self::entity_to_data))
    }

    async fn list(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<StatusHistoryData>, u64)> {
        let paginator = StatusHistory::find()
            .order_by_desc(status_history::Column::CreatedAt)
            .paginate(&*self.db, limit);

        let total = paginator.num_items().await?;
        let entities = paginator.fetch_page(offset / limit).await?;

        let data: Vec<StatusHistoryData> = entities
            .into_iter()
            .map(Self::entity_to_data)
            .collect();

        Ok((data, total))
    }
    
    async fn list_by_date_range(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<StatusHistoryData>, u64)> {
        let paginator = StatusHistory::find()
            .filter(status_history::Column::StartDate.gte(start_date.naive_local()))
            .filter(status_history::Column::EndDate.lte(end_date.naive_local()))
            .order_by_desc(status_history::Column::StartDate)
            .paginate(&*self.db, limit);

        let total = paginator.num_items().await?;
        let entities = paginator.fetch_page(offset / limit).await?;

        let data: Vec<StatusHistoryData> = entities
            .into_iter()
            .map(Self::entity_to_data)
            .collect();

        Ok((data, total))
    }

    async fn update_status(&self, id: i32, status: Status) -> Result<()> {
        let entity = StatusHistory::find_by_id(id)
            .one(&*self.db)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let mut active_model: status_history::ActiveModel = entity.into();
        active_model.status = Set(status);

        StatusHistory::update(active_model)
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    async fn delete(&self, id: i32) -> Result<()> {
        StatusHistory::delete_by_id(id).exec(&*self.db).await?;
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
    async fn test_create_status_history() {
        let db = create_test_db().await;
        let repo = DbStatusHistoryRepository::new(Arc::new(db));

        let data = CreateStatusHistory {
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            count: 100,
            status: Status::Clean,
        };

        let result = repo.create(data).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.count, 100);
        assert_eq!(created.status, Status::Clean);
    }

    #[tokio::test]
    async fn test_find_by_date_range() {
        let db = create_test_db().await;
        let repo = DbStatusHistoryRepository::new(Arc::new(db));

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();

        let data = CreateStatusHistory {
            start_date,
            end_date,
            count: 200,
            status: Status::Immaculate,
        };

        repo.create(data.clone()).await.unwrap();

        let found = repo.find_by_date_range(start_date, end_date).await.unwrap();
        assert!(found.is_some());

        let history = found.unwrap();
        assert_eq!(history.count, 200);
        assert_eq!(history.status, Status::Immaculate);
    }

    #[tokio::test]
    async fn test_get_latest() {
        let db = create_test_db().await;
        let repo = DbStatusHistoryRepository::new(Arc::new(db));

        let data1 = CreateStatusHistory {
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 21).unwrap(),
            count: 300,
            status: Status::Spotless,
        };

        repo.create(data1).await.unwrap();

        let latest = repo.get_latest().await.unwrap();
        assert!(latest.is_some());

        let history = latest.unwrap();
        assert_eq!(history.status, Status::Spotless);
    }
}
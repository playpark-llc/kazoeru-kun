use std::sync::Arc;
use chrono::{DateTime, Local, TimeZone};
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::domain::{calculate_rank_change, apply_rank_change, get_current_week_range};
use crate::repositories::{
    SessionCounterRepository, StatusHistoryRepository, 
    CreateStatusHistory, RepositoryError
};
use crate::sea_generated::sea_orm_active_enums::Status;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateResult {
    pub week_start: DateTime<Local>,
    pub week_end: DateTime<Local>,
    pub weekly_count: i64,
    pub target_value: i64,
    pub previous_status: String,  // Status型をStringに変更
    pub new_status: String,       // Status型をStringに変更
    pub rank_change: i32,
    pub message: String,
    pub was_duplicate: bool,
}

pub struct StatusService {
    status_history_repo: Arc<dyn StatusHistoryRepository>,
    counter_repo: Arc<dyn SessionCounterRepository>,
}

impl StatusService {
    pub fn new(
        status_history_repo: Arc<dyn StatusHistoryRepository>,
        counter_repo: Arc<dyn SessionCounterRepository>,
    ) -> Self {
        Self {
            status_history_repo,
            counter_repo,
        }
    }

    /// 週次ステータスを更新
    pub async fn update_weekly_status(&self, target_value: i64) -> Result<StatusUpdateResult, RepositoryError> {
        // 今週の期間を取得
        let (week_start, week_end) = get_current_week_range();
        
        // ローカルタイムゾーンに変換
        let start_date_local = Local.from_utc_datetime(&week_start.naive_utc());
        let end_date_local = Local.from_utc_datetime(&week_end.naive_utc());
        
        // 今週のカウント合計を計算
        let weekly_count = self.counter_repo
            .sum_count_between(start_date_local, end_date_local)
            .await?;
        
        log::debug!("Weekly count: {}, Target: {}", weekly_count, target_value);
        
        // 現在のステータスを取得（最新のエントリ）
        let current_status_result = self.status_history_repo
            .get_latest()
            .await?;
        
        let current_status = current_status_result
            .map(|r| r.status)
            .unwrap_or(Status::Clean); // 前回のステータスがない場合はCleanから開始
        
        // ランクの変化を計算
        let rank_change = calculate_rank_change(weekly_count, target_value);
        
        // 新しいステータスを適用
        let new_status = apply_rank_change(current_status.clone(), rank_change);
        
        log::debug!(
            "Current status: {:?}, Rank change: {}, New status: {:?}",
            current_status,
            rank_change,
            new_status
        );
        
        // 今週の記録が既に存在するかチェック
        let existing_records = self.status_history_repo
            .find_by_date_range(start_date_local, end_date_local)
            .await?;
        
        let existing_record = existing_records.into_iter().find(|r| 
            r.start_date == start_date_local.naive_local().date() && 
            r.end_date == end_date_local.naive_local().date()
        );
        
        // 重複チェック
        if let Some(existing) = existing_record {
            log::info!(
                "Status history duplicate attempt - Period: {} to {}, Status: {:?}, Count: {}",
                start_date_local.naive_local(),
                end_date_local.naive_local(),
                existing.status,
                existing.count
            );
            
            let existing_status = existing.status.clone();
            return Ok(StatusUpdateResult {
                week_start: start_date_local,
                week_end: end_date_local,
                weekly_count,
                target_value,
                previous_status: format!("{:?}", current_status),
                new_status: format!("{:?}", existing_status),
                rank_change,
                message: format!(
                    "Status history already exists for this period (existing status: {:?}, count: {}, target: {})",
                    existing_status, weekly_count, target_value
                ),
                was_duplicate: true,
            });
        }
        
        // 新しいステータス履歴レコードを挿入
        let new_record = CreateStatusHistory {
            start_date: start_date_local.naive_local().date(),
            end_date: end_date_local.naive_local().date(),
            count: Decimal::from(weekly_count),
            status: new_status.clone(),
        };
        
        self.status_history_repo.create(new_record).await?;
        
        let message = format!(
            "Status updated from {:?} to {:?} (count: {}, target: {}, change: {})",
            current_status,
            new_status,
            weekly_count,
            target_value,
            if rank_change > 0 {
                format!("+{}", rank_change)
            } else {
                rank_change.to_string()
            }
        );
        
        Ok(StatusUpdateResult {
            week_start: start_date_local,
            week_end: end_date_local,
            weekly_count,
            target_value,
            previous_status: format!("{:?}", current_status),
            new_status: format!("{:?}", new_status),
            rank_change,
            message,
            was_duplicate: false,
        })
    }

    /// 現在のステータスを取得
    pub async fn get_current_status(&self) -> Result<Option<Status>, RepositoryError> {
        let status_data = self.status_history_repo.get_latest().await?;
        Ok(status_data.map(|s| s.status))
    }

    /// 指定期間のステータス履歴を取得
    pub async fn get_status_history(
        &self,
        start_date: DateTime<Local>,
        end_date: DateTime<Local>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<crate::repositories::status_history::StatusHistoryData>, u64), RepositoryError> {
        self.status_history_repo
            .list_by_date_range(start_date, end_date, limit, offset)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // テスト用のモックリポジトリ
    struct MockStatusHistoryRepository {
        records: Arc<Mutex<Vec<crate::repositories::status_history::StatusHistoryData>>>,
        next_id: Arc<Mutex<i32>>,
    }

    impl MockStatusHistoryRepository {
        fn new() -> Self {
            Self {
                records: Arc::new(Mutex::new(Vec::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }
    }

    #[async_trait]
    impl StatusHistoryRepository for MockStatusHistoryRepository {
        async fn create(&self, data: CreateStatusHistory) -> Result<crate::repositories::status_history::StatusHistoryData, RepositoryError> {
            let mut records = self.records.lock().unwrap();
            let mut next_id = self.next_id.lock().unwrap();
            
            let id = *next_id;
            *next_id += 1;
            
            let record = crate::repositories::status_history::StatusHistoryData {
                id,
                start_date: data.start_date,
                end_date: data.end_date,
                count: data.count,
                status: data.status,
                created_at: Local::now(),
                updated_at: Local::now(),
            };
            
            records.push(record.clone());
            Ok(record)
        }

        async fn find_by_id(&self, _id: i32) -> Result<Option<crate::repositories::status_history::StatusHistoryData>, RepositoryError> {
            unimplemented!()
        }

        async fn find_by_date_range(&self, start: DateTime<Local>, end: DateTime<Local>) -> Result<Vec<crate::repositories::status_history::StatusHistoryData>, RepositoryError> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .filter(|r| {
                    let record_start = Local.from_local_datetime(&r.start_date.and_hms_opt(0, 0, 0).unwrap()).single().unwrap_or(start);
                    let record_end = Local.from_local_datetime(&r.end_date.and_hms_opt(0, 0, 0).unwrap()).single().unwrap_or(end);
                    record_start == start && record_end == end
                })
                .cloned()
                .collect())
        }

        async fn list(&self, _limit: u64, _offset: u64) -> Result<(Vec<crate::repositories::status_history::StatusHistoryData>, u64), RepositoryError> {
            let records = self.records.lock().unwrap();
            Ok((records.clone(), records.len() as u64))
        }

        async fn list_by_date_range(&self, _start: DateTime<Local>, _end: DateTime<Local>, limit: u64, offset: u64) -> Result<(Vec<crate::repositories::status_history::StatusHistoryData>, u64), RepositoryError> {
            let records = self.records.lock().unwrap();
            let total = records.len() as u64;
            let start = offset as usize;
            let end = (offset + limit) as usize;
            Ok((records[start.min(records.len())..end.min(records.len())].to_vec(), total))
        }

        async fn get_latest(&self) -> Result<Option<crate::repositories::status_history::StatusHistoryData>, RepositoryError> {
            let records = self.records.lock().unwrap();
            Ok(records.last().cloned())
        }

        async fn update_status(&self, _id: i32, _status: Status) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: i32) -> Result<(), RepositoryError> {
            unimplemented!()
        }
    }

    struct MockSessionCounterRepository {
        counts: Arc<Mutex<HashMap<String, i32>>>,
    }

    impl MockSessionCounterRepository {
        fn new() -> Self {
            Self {
                counts: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl SessionCounterRepository for MockSessionCounterRepository {
        async fn upsert(&self, _data: crate::repositories::CreateSessionCounter) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn upsert_many(&self, _data: Vec<crate::repositories::CreateSessionCounter>) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn upsert_many_with_txn(&self, _data: Vec<crate::repositories::CreateSessionCounter>, _txn: &sea_orm::DatabaseTransaction) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn sum_count_between(&self, _start_date: DateTime<Local>, _end_date: DateTime<Local>) -> Result<i64, RepositoryError> {
            // テスト用に固定値を返す
            Ok(1000)
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


        async fn delete(&self, _id: i32) -> Result<(), RepositoryError> {
            unimplemented!()
        }

        async fn delete_by_session_id(&self, _session_id: &str) -> Result<(), RepositoryError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_update_weekly_status_first_time() {
        let status_repo = Arc::new(MockStatusHistoryRepository::new());
        let counter_repo = Arc::new(MockSessionCounterRepository::new());
        let service = StatusService::new(status_repo.clone(), counter_repo);

        let result = service.update_weekly_status(1500).await.unwrap();
        
        assert_eq!(result.weekly_count, 1000);
        assert_eq!(result.target_value, 1500);
        assert_eq!(result.previous_status, "Clean"); // 初回はCleanから開始
        assert!(!result.was_duplicate);
    }

    #[tokio::test]
    async fn test_duplicate_status_update() {
        let status_repo = Arc::new(MockStatusHistoryRepository::new());
        let counter_repo = Arc::new(MockSessionCounterRepository::new());
        let service = StatusService::new(status_repo.clone(), counter_repo);

        // 最初の更新
        let result1 = service.update_weekly_status(1500).await.unwrap();
        assert!(!result1.was_duplicate);
        
        // 重複更新（同じ週）
        let result2 = service.update_weekly_status(1500).await.unwrap();
        assert!(result2.was_duplicate);
    }
}
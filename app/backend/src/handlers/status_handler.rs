use rocket::{get, put, serde::json::Json, State};
use std::sync::Arc;
use crate::config::AppConfig;
use crate::models::{
    domain::status_to_string,
    response::{ErrorResponse, PageInfo, Response, StatusHistoryRecord, StatusHistoryResponse},
};
use crate::repositories::{SessionCounterRepository, StatusHistoryRepository};
use crate::services::StatusService;
use crate::{DateRange, get_date_range_local};

/// PUT /status_history - ステータス履歴の更新
#[put("/status_history")]
pub async fn update_status(
    status_history_repo: &State<Arc<dyn StatusHistoryRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    config: &State<AppConfig>,
) -> Result<Json<Response<String>>, Json<ErrorResponse>> {
    // ビジネスロジックはサービス層に委譲
    let status_service = StatusService::new(
        status_history_repo.inner().clone(),
        counter_repo.inner().clone(),
    );
    
    match status_service.update_weekly_status(config.target_value).await {
        Ok(update_result) => {
            Ok(Json(Response {
                message: update_result.message,
                result: "Success".to_string(),
            }))
        }
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to update status: {:?}", e),
        })),
    }
}

/// GET /status_history - ステータス履歴の取得（ページネーション対応）
#[get("/status_history?<limit>&<offset>&<date_range..>")]
pub async fn get_history(
    status_history_repo: &State<Arc<dyn StatusHistoryRepository>>,
    counter_repo: &State<Arc<dyn SessionCounterRepository>>,
    limit: Option<u64>,
    offset: Option<u64>,
    date_range: Option<DateRange>,
) -> Result<Json<Response<StatusHistoryResponse>>, Json<ErrorResponse>> {
    let status_service = StatusService::new(
        status_history_repo.inner().clone(),
        counter_repo.inner().clone(),
    );
    
    // Get the current status (most recent entry)
    let current_status = status_history_repo
        .get_latest()
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: format!("Failed to get current status: {:?}", e),
            })
        })?;

    // Convert current status to response format
    let current_status_record = current_status.map(|s| StatusHistoryRecord {
        id: s.id,
        start_date: s.start_date,
        end_date: s.end_date,
        count: s.count,
        status: status_to_string(s.status),
        created_at: s.created_at.with_timezone(&chrono::Utc),
    });

    // Apply pagination parameters
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    // Get historical records with pagination
    let (history_records, total_count) = if let Some(range) = date_range {
        let date_range = get_date_range_local(Some(range));
        status_service
            .get_status_history(date_range.start_date, date_range.end_date, limit, offset)
            .await
            .map_err(|e| {
                Json(ErrorResponse {
                    error: format!("Failed to get status history: {:?}", e),
                })
            })?
    } else {
        status_history_repo
            .list(limit, offset)
            .await
            .map_err(|e| {
                Json(ErrorResponse {
                    error: format!("Failed to get status history: {:?}", e),
                })
            })?
    };

    // Convert to response format
    let history: Vec<StatusHistoryRecord> = history_records
        .into_iter()
        .map(|h| StatusHistoryRecord {
            id: h.id,
            start_date: h.start_date,
            end_date: h.end_date,
            count: h.count,
            status: status_to_string(h.status),
            created_at: h.created_at.with_timezone(&chrono::Utc),
        })
        .collect();

    let filtered_records = history.len();

    let response = StatusHistoryResponse {
        current_status: current_status_record,
        history,
        total_records: filtered_records,
        total_count: total_count as usize,
        page_info: PageInfo {
            limit,
            offset,
            has_more: (offset as usize + filtered_records) < total_count as usize,
        },
    };

    Ok(Json(Response {
        message: "ステータス履歴を取得しました".to_string(),
        result: response,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use sea_orm::prelude::Decimal;
    use crate::sea_generated::sea_orm_active_enums::Status;

    #[test]
    fn test_status_history_response_structure() {
        // Test that StatusHistoryRecord can be created
        let record = StatusHistoryRecord {
            id: 1,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            count: Decimal::from(1000),
            status: "Clean".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(record.id, 1);
        assert_eq!(record.status, "Clean");

        // Test that StatusHistoryResponse can be created with pagination
        let response = StatusHistoryResponse {
            current_status: Some(record.clone()),
            history: vec![record],
            total_records: 1,
            total_count: 1,
            page_info: PageInfo {
                limit: 100,
                offset: 0,
                has_more: false,
            },
        };

        assert!(response.current_status.is_some());
        assert_eq!(response.total_records, 1);
        assert_eq!(response.total_count, 1);
        assert_eq!(response.history.len(), 1);
        assert_eq!(response.page_info.limit, 100);
        assert_eq!(response.page_info.offset, 0);
        assert!(!response.page_info.has_more);
    }

    #[test]
    fn test_get_status_history_endpoint_params() {
        // Test that the endpoint accepts the correct parameters
        // The actual endpoint testing would require database mocking

        // Test parameter validation
        let limit: Option<u64> = Some(10);
        let offset: Option<u64> = Some(0);
        assert_eq!(limit.unwrap_or(100), 10);
        assert_eq!(offset.unwrap_or(0), 0);

        // Test default values
        let limit: Option<u64> = None;
        let offset: Option<u64> = None;
        assert_eq!(limit.unwrap_or(100), 100);
        assert_eq!(offset.unwrap_or(0), 0);
    }

    #[test]
    fn test_pagination_logic() {
        // Test pagination calculation
        let total_count = 100;
        let _limit = 10;
        let offset = 20;
        let filtered_records = 10;

        let has_more = (offset as usize + filtered_records) < total_count;
        assert!(has_more);

        // Test when no more records
        let offset = 95;
        let filtered_records = 5;
        let has_more = (offset as usize + filtered_records) < total_count;
        assert!(!has_more);
    }

    #[test]
    fn test_status_history_record_with_proper_status() {
        // Test that StatusHistoryRecord uses proper status format
        let record = StatusHistoryRecord {
            id: 1,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            count: Decimal::from(1000),
            status: status_to_string(Status::Clean),
            created_at: Utc::now(),
        };

        // Should be uppercase format now
        assert_eq!(record.status, "CLEAN");
    }

    #[test]
    fn test_page_info_structure() {
        let page_info = PageInfo {
            limit: 50,
            offset: 10,
            has_more: true,
        };
        
        assert_eq!(page_info.limit, 50);
        assert_eq!(page_info.offset, 10);
        assert!(page_info.has_more);
    }
}
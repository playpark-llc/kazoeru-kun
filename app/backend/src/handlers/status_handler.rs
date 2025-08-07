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
use chrono::NaiveDate;
use rocket::serde::Serialize;
use sea_orm::{prelude::Decimal, FromQueryResult};

/// Response wrapper for successful API responses
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response<T> {
    pub message: String,
    pub result: T,
}

/// Response for error cases
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub error: String,
}

/// Session count response
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SessionCount {
    pub count: u32,
}

/// Session counter summary from database aggregation
#[derive(FromQueryResult, Debug, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SessionCounterSummary {
    pub sum_count: i64,
}

/// Status history record
#[derive(Debug, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct StatusHistoryRecord {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub count: Decimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Pagination information
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PageInfo {
    pub limit: u64,
    pub offset: u64,
    pub has_more: bool,
}

/// Status history response with pagination
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatusHistoryResponse {
    pub current_status: Option<StatusHistoryRecord>,
    pub history: Vec<StatusHistoryRecord>,
    pub total_records: usize, // Number of records in current page
    pub total_count: usize,   // Total number of records matching the query
    pub page_info: PageInfo,
}

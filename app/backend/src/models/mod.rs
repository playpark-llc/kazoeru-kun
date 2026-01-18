/// Model layer for Kazoeru-kun backend
/// Provides clean separation of request, response, and domain models
pub mod domain;
pub mod request;
pub mod response;

// Re-export commonly used types for convenience
pub use domain::{
    apply_rank_change, calculate_rank_change, get_current_week_range, status_to_string,
    StatusCalculation,
};
pub use request::{PaginationQuery, SessionData, SessionRequest};
pub use response::{
    ErrorResponse, PageInfo, Response, SessionCount, SessionCounterSummary, StatusHistoryRecord,
    StatusHistoryResponse,
};

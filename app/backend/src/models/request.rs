use rocket::form::FromForm;

/// Pagination parameters for API requests
#[derive(FromForm)]
pub struct PaginationQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Session request model for API endpoints
pub struct SessionRequest {
    pub session_id: String,
    pub count: u32,
}

/// Session data model
pub struct SessionData {
    pub session_id: String,
    pub count: i32,
    pub expiration: chrono::DateTime<chrono::Local>,
}

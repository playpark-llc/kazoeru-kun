use rocket::{Route, routes};
use crate::handlers::{session_handler, status_handler};

/// ルーティング定義を集約
pub fn configure_routes() -> Vec<Route> {
    routes![
        // Session related endpoints
        session_handler::create_session,
        session_handler::sync_sessions,
        session_handler::get_session_counter,
        session_handler::get_count_summary,
        // Status related endpoints
        status_handler::update_status,
        status_handler::get_history,
    ]
}
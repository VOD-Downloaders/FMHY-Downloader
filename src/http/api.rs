use std::sync::{Arc, Mutex};

use serde_json::json;
use axum::{
    extract,
    extract::State,
    response,
    http::{header, StatusCode},
};

use super::bodies::*;

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct AppState {
    a: i8,
}

impl AppState {
    pub fn new() -> Self {
        Self { a: 0 }
    }
}

/////////////////////////////////////////////////////
// API
/////////////////////////////////////////////////////
pub async fn post_download(
    State(state): State<Arc<Mutex<AppState>>>, extract::Json(payload): extract::Json<DownloadRequest>,
) -> Result<DownloadResponse, ErrorResponse> {
    trace!("{:?}", payload);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR".to_string(),
    })
}

use std::sync::{Arc, Mutex};

use serde_json::json;
use axum::{
    extract,
    extract::{State, Path},
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
    trace!("{}, {}", payload.input_url, payload.output_file);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR".to_string(),
    })
}

pub async fn get_download_status(
    State(state): State<Arc<Mutex<AppState>>>, Path(path): Path<DownloadStatusPath>,
) -> Result<DownloadStatusResponse, ErrorResponse> {
    trace!("{}", path.id);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR 2".to_string(),
    })
}

use serde::{Deserialize, Serialize};
use axum::{
    response::{self, IntoResponse},
    http::StatusCode,
};

/////////////////////////////////////////////////////
// Requests
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub input_url: String,
    pub output_file: String,
}

/////////////////////////////////////////////////////
// Paths
/////////////////////////////////////////////////////
#[derive(Deserialize)]
pub struct DownloadStatusPath {
    pub id: u64,
}

/////////////////////////////////////////////////////
// Responses
/////////////////////////////////////////////////////
#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct DownloadResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for DownloadResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct DownloadStatusResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub status_message: String,
}

impl IntoResponse for DownloadStatusResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

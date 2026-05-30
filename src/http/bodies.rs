use serde::{Deserialize, Serialize};
use axum::{
    response::{self, IntoResponse},
    http::StatusCode,
};

use super::super::download;
use super::super::config::Indexer;
use super::super::config::IndexerSpecification;

/////////////////////////////////////////////////////
// Requests
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub indexer_name: String,
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
pub struct IndexersResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub indexers: Vec<Indexer>,
}

impl IntoResponse for IndexersResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct IndexerSpecificationsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub indexers: Vec<IndexerSpecification>,
}

impl IntoResponse for IndexerSpecificationsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct DownloadResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub id: u64,
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
    #[serde(rename = "status")]
    pub status_object: download::DownloadStatus,
}

impl IntoResponse for DownloadStatusResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

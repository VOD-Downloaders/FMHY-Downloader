use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use axum::{
    response::{self, IntoResponse},
    http::StatusCode,
};

use super::super::config::Indexer;
use super::super::config::IndexerSpecification;
use super::super::streams::Stream;

/////////////////////////////////////////////////////
// Requests
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct CreateIndexerRequest {
    pub indexer: Indexer,
}

#[derive(Debug, Deserialize)]
pub struct StreamsRequest {
    pub indexer_name: String,
    pub input_url: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub indexer_name: String,
    pub stream: Stream,
    pub output_file: String,
}

/////////////////////////////////////////////////////
// Paths
/////////////////////////////////////////////////////
#[derive(Deserialize)]
pub struct DownloadStatusPath {
    pub id: u32,
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
pub struct CreateIndexerResponse {
    #[serde(skip)]
    pub status: StatusCode,
}

impl IntoResponse for CreateIndexerResponse {
    fn into_response(self) -> response::Response {
        (self.status).into_response()
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
pub struct StreamsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub streams: Vec<Stream>,
}

impl IntoResponse for StreamsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct DownloadResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub id: u32,
}

impl IntoResponse for DownloadResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

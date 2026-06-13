use serde::{Deserialize, Serialize};
use axum::{
    response::{self, IntoResponse},
    http::StatusCode,
};

use super::super::config::Indexer;
use super::super::config::IndexerSpecification;
use super::super::search::Stream;
use super::super::search::MovieResultBody;
use super::super::search::SeriesResultBody;

/////////////////////////////////////////////////////
// Requests
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct CreateIndexerRequest {
    pub indexer: Indexer,
}

#[derive(Debug, Deserialize)]
pub struct DeleteIndexerRequest {
    pub name: String,
}

/////////////////////////////////////////////////////
// Queries
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct SearchMovieQuery {
    pub name: String,
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchSeriesQuery {
    pub name: String,
    pub page: i32,
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
pub struct DeleteIndexerResponse {
    #[serde(skip)]
    pub status: StatusCode,
}

impl IntoResponse for DeleteIndexerResponse {
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
pub struct SearchMovieResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: MovieResultBody,
}

impl IntoResponse for SearchMovieResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct SearchSeriesResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: SeriesResultBody,
}

impl IntoResponse for SearchSeriesResponse {
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

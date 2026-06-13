use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use url::Url;
use axum::{
    extract,
    extract::{State, Query},
    http::{StatusCode},
};

use super::bodies::*;
use super::super::env;
use super::super::config;
use super::super::request;
use super::super::search;
use super::super::download;

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct DownloadInfo {}

pub struct AppState {
    pub state: RwLock<config::State>,
    pub environment: env::EnvOptions, // readonly
    pub downloads: RwLock<HashMap<u32, DownloadInfo>>,
}

impl AppState {
    pub fn new(environment: env::EnvOptions, state: config::State) -> Self {
        Self {
            state: RwLock::new(state),
            environment: environment,
            downloads: RwLock::new(HashMap::new()),
        }
    }
}

/////////////////////////////////////////////////////
// API
/////////////////////////////////////////////////////
pub async fn get_indexers(State(state): State<Arc<AppState>>) -> Result<IndexersResponse, ErrorResponse> {
    trace!("Received get_indexers");

    Ok(IndexersResponse {
        status: StatusCode::OK,
        indexers: state.state.read().unwrap().indexers.clone(),
    })
}

pub async fn post_create_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<CreateIndexerRequest>,
) -> Result<CreateIndexerResponse, ErrorResponse> {
    trace!("Received post_create_indexer for {:?}", payload);

    config::write_indexer_to_file(&payload.indexer, config::indexer_name_to_path(payload.indexer.name.as_str()).as_path())
        .await
        .map_err(|error| ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: format!("Unable to write indexer to file due to error: {}", error),
        })?;

    // Update indexers in state
    state.state.write().unwrap().indexers = config::load_indexers().await;

    Ok(CreateIndexerResponse { status: StatusCode::OK })
}

pub async fn post_delete_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<DeleteIndexerRequest>,
) -> Result<DeleteIndexerResponse, ErrorResponse> {
    trace!("Received post_delete_indexer for \"{}\".", payload.name);

    tokio::fs::remove_file(config::indexer_name_to_path(payload.name.as_str()).as_path())
        .await
        .map_err(|error| ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: format!("Unable to delete indexer \"{}\" due to error: {}", payload.name, error),
        })?;

    // Update indexers in state
    state.state.write().unwrap().indexers = config::load_indexers().await;

    Ok(DeleteIndexerResponse { status: StatusCode::OK })
}

pub async fn get_indexer_specifications(State(_state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received get_indexer_specifications");

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: config::load_indexer_specifications().await,
    })
}

pub async fn post_refresh_indexer_specifications(State(_state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received post_refresh_indexer_specifications");

    config::get_new_specifications().await.map_err(|error| ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: format!("Unable to retrieve latest indexer specifications due to error: {}", error),
    })?;

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: config::load_indexer_specifications().await,
    })
}

pub async fn get_search_movie(
    State(_state): State<Arc<AppState>>, Query(query): Query<SearchMovieQuery>,
) -> Result<SearchMovieResponse, ErrorResponse> {
    trace!("Received post_search_movie for \"{}\" on page {}.", query.name, query.page);

    // TODO: ...
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::tmdb_get_movies(query.name.as_str(), Some(query.page), &requester).await;

    Ok(SearchMovieResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_search_series(
    State(_state): State<Arc<AppState>>, Query(query): Query<SearchSeriesQuery>,
) -> Result<SearchSeriesResponse, ErrorResponse> {
    trace!("Received post_search_series for \"{}\" on page {}.", query.name, query.page);

    // TODO: ...
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::tmdb_get_series(query.name.as_str(), Some(query.page), &requester).await;

    Ok(SearchSeriesResponse {
        status: StatusCode::OK,
        response: response,
    })
}

use std::{
    collections::HashMap,
    fmt::format,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use url::Url;
use axum::{
    extract,
    extract::{State, Path},
    http::{StatusCode},
};

use super::bodies::*;
use super::super::env;
use super::super::config;
use super::super::request;
use super::super::streams;
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

pub async fn get_indexer_specifications(State(_state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received get_indexer_specifications");

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: config::load_indexer_specifications().await,
    })
}

pub async fn post_streams(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<StreamsRequest>,
) -> Result<StreamsResponse, ErrorResponse> {
    trace!("Received get_streams for {}.", payload.input_url);

    let indexer = {
        let guard = state.state.read().unwrap();
        let indexer = guard
            .indexers
            .iter()
            .find(|item| item.name == payload.indexer_name)
            .ok_or(ErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: format!("Indexer by name \"{}\" not found.", payload.indexer_name),
            })?;

        indexer.clone()
    };

    let url = Url::parse(payload.input_url.as_str()).map_err(|error| ErrorResponse {
        status: StatusCode::BAD_REQUEST,
        error: format!("Invalid URL passed in, error: {}", error),
    })?;

    // TODO: Handle cloudflare
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::PRECONDITION_FAILED,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    // let flaresolverr_url = state.environment.flaresolverr_url.clone().unwrap();
    //
    // let requester =
    //     request::Requester::get_flaresolvedd_native(request::RequesterSpecification::default(), &flaresolverr_url, &url).map_err(|error| {
    //         ErrorResponse {
    //             status: StatusCode::PRECONDITION_FAILED,
    //             error: format!("Unable to create requester object due to error: {}", error),
    //         }
    //     })?;

    let streams = streams::get_streams(&indexer, &requester, &url).await;

    Ok(StreamsResponse {
        status: StatusCode::OK,
        streams: streams,
    })
}

pub async fn post_download(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<DownloadRequest>,
) -> Result<DownloadResponse, ErrorResponse> {
    trace!("Received post_download with: {:?}", payload);

    let indexer = {
        let guard = state.state.read().unwrap();
        let indexer = guard
            .indexers
            .iter()
            .find(|item| item.name == payload.indexer_name)
            .ok_or(ErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: format!("Indexer by name \"{}\" not found.", payload.indexer_name),
            })?;

        indexer.clone()
    };

    // TODO: Handle cloudflare
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::PRECONDITION_FAILED,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let output_file = PathBuf::from(payload.output_file);

    tokio::spawn(async move {
        let result = download::download_stream(&indexer, payload.stream, &requester, output_file.as_path()).await;

        if let Err(error) = result {
            error!("Download failed due to error: {}", error);
        }
    });

    let id = rand::random::<u32>();
    trace!("Adding download by id {} to active downloads...", id);
    {
        let mut guard = state.downloads.write().unwrap();
        guard.insert(id, DownloadInfo {});
    }

    Ok(DownloadResponse {
        status: StatusCode::OK,
        id: id,
    })
}

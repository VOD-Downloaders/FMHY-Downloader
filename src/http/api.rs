use std::{
    collections::HashMap,
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
use super::super::download;

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct DownloadInfo {
    pub handle: std::thread::JoinHandle<()>,
}

pub struct AppState {
    pub state: RwLock<config::State>,
    pub environment: env::EnvOptions, // readonly
    pub downloads: RwLock<HashMap<u64, DownloadInfo>>,
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

pub async fn post_download(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<DownloadRequest>,
) -> Result<DownloadResponse, ErrorResponse> {
    trace!("Received post_download with: {:?}", payload);

    let state_clone = Arc::clone(&state);

    let url = Url::parse(payload.input_url.as_str()).map_err(|_error| ErrorResponse {
        status: StatusCode::BAD_REQUEST,
        error: "Invalid URL passed in".to_string(),
    })?;

    let output_path = PathBuf::from(payload.output_file);

    let download_status = Arc::new(RwLock::new(download::DownloadStatus::Starting));
    let download_status_clone = Arc::clone(&download_status);

    // TODO: Resolve indexer from request
    let (download_method, uses_cloudflare) = {
        let guard = state_clone.state.read().unwrap();
        let indexer = guard.indexers.get(0).unwrap();
        (indexer.download.clone(), indexer.uses_cloudflare)
    };

    tokio::spawn(async move {
        let result = download::download_file(
            &download_method,
            &state_clone.environment.flaresolverr_url,
            download_status_clone,
            &url,
            output_path.as_path(),
            uses_cloudflare,
        )
        .await;

        // TODO: Result to download_status
    });

    // TODO: Lock again and put state in AppState

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR".to_string(),
    })
}

pub async fn get_download_status(
    State(state): State<Arc<AppState>>, Path(path): Path<DownloadStatusPath>,
) -> Result<DownloadStatusResponse, ErrorResponse> {
    trace!("Received get_download_status for {}", path.id);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR 2".to_string(),
    })
}

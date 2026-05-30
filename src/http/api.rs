use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
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
use super::super::download;

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct DownloadInfo {
    pub handle: std::thread::JoinHandle<()>,
}

pub struct AppState {
    pub state: Arc<config::State>,
    pub environment: Arc<env::EnvOptions>,
    pub downloads: HashMap<u64, DownloadInfo>,
}

impl AppState {
    pub fn new(environment: env::EnvOptions, state: config::State) -> Self {
        Self {
            state: Arc::new(state),
            environment: Arc::new(environment),
            downloads: HashMap::new(),
        }
    }
}

/////////////////////////////////////////////////////
// API
/////////////////////////////////////////////////////
pub async fn get_indexers(State(state): State<Arc<Mutex<AppState>>>) -> Result<IndexersResponse, ErrorResponse> {
    trace!("Received get_indexers");

    let config_state: Arc<config::State> = {
        let state = state.lock().unwrap();
        Arc::clone(&state.state)
    };

    Ok(IndexersResponse {
        status: StatusCode::OK,
        indexers: config_state.indexers.clone(),
    })
}

pub async fn post_download(
    State(state): State<Arc<Mutex<AppState>>>, extract::Json(payload): extract::Json<DownloadRequest>,
) -> Result<DownloadResponse, ErrorResponse> {
    trace!("Received post_download with: {:?}", payload);

    let environment: Arc<env::EnvOptions> = {
        let state = state.lock().unwrap();
        Arc::clone(&state.environment)
    };

    let url = Url::parse(payload.input_url.as_str()).map_err(|_error| ErrorResponse {
        status: StatusCode::BAD_REQUEST,
        error: "Invalid URL passed in".to_string(),
    })?;

    let referer = {
        let scheme = url.scheme();
        let host = url.host_str().ok_or(ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: "Failed to retrieve domain from URL".to_string(),
        })?;

        Url::parse(&format!("{}://{}", scheme, host)).map_err(|_| ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: "Failed to retrieve domain from URL".to_string(),
        })?
    };

    tokio::spawn(async move {
        let credentials = request::get_credentials(&environment.flaresolverr_url, &referer).await.unwrap();

        let index_data = download::get_index(&environment, &url, &credentials).await.unwrap();

        let path = std::path::PathBuf::from(payload.output_file);
        let _ = download::download_file(&environment, &credentials, index_data, path.as_path())
            .await
            .unwrap();
    });

    // TODO: Lock again and put state in AppState

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR".to_string(),
    })
}

pub async fn get_download_status(
    State(state): State<Arc<Mutex<AppState>>>, Path(path): Path<DownloadStatusPath>,
) -> Result<DownloadStatusResponse, ErrorResponse> {
    trace!("Received get_download_status for {}", path.id);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR 2".to_string(),
    })
}

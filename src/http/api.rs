use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde_json::json;
use axum::{
    extract,
    extract::{State, Path},
    response,
    http::{header, StatusCode},
};

use super::bodies::*;
use super::super::env;
use super::super::request;
use super::super::downloader;

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct DownloadInfo {
    pub handle: std::thread::JoinHandle<()>,
}

pub struct AppState {
    pub environment: Arc<env::EnvOptions>,
    pub downloads: HashMap<u64, DownloadInfo>,
}

impl AppState {
    pub fn new(environment: env::EnvOptions) -> Self {
        Self {
            environment: Arc::new(environment),
            downloads: HashMap::new(),
        }
    }
}

/////////////////////////////////////////////////////
// API
/////////////////////////////////////////////////////
pub async fn post_download(
    State(state): State<Arc<Mutex<AppState>>>, extract::Json(payload): extract::Json<DownloadRequest>,
) -> Result<DownloadResponse, ErrorResponse> {
    trace!("Received post_download with: {:?}", payload);

    let environment: Arc<env::EnvOptions> = {
        let state = state.lock().unwrap();
        Arc::clone(&state.environment)
    };

    tokio::spawn(async move {
        let credentials = request::get_credentials(environment.flaresolverr_url.as_str(), "https://www.cineby.sc/")
            .await
            .unwrap(); // TODO: Auto generate

        let index_data = downloader::get_index(&environment, payload.input_url.as_str(), &credentials)
            .await
            .unwrap();

        let path = std::path::PathBuf::from(payload.output_file);
        let _ = downloader::download_file(&environment, &credentials, index_data, path.as_path());
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
    trace!("{}", path.id);

    Err(ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: "MY CUSTOM ERROR 2".to_string(),
    })
}

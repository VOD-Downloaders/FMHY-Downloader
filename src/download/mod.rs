use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use thiserror::Error;
use url::Url;
use serde::Serialize;

mod index_intercept;
use index_intercept as index;
mod master_intercept;
use master_intercept as master;
// mod mp4_intercept;
// use mp4_intercept as mp4;

use super::request;
use super::config::DownloadMethod;
use super::config::DownloadSpecification;
use super::config::ProcessingSpecification;

pub const CHROMIUM_PATH: &str = "/usr/lib/chromium/chromium";

/////////////////////////////////////////////////////
// DownloadArguments
/////////////////////////////////////////////////////
pub struct IndexInterceptArguments {
    pub preprocessing: ProcessingSpecification,
    pub index_attempts: u8,
    pub index_wait_time: u8,
    pub segment_attempts: u8,
    pub segment_timeout: u8,
}

impl Default for IndexInterceptArguments {
    fn default() -> Self {
        Self {
            preprocessing: ProcessingSpecification::default(),
            index_attempts: 5,
            index_wait_time: 6,
            segment_attempts: 3,
            segment_timeout: 5,
        }
    }
}

pub struct MasterInterceptArguments {
    pub preprocessing: ProcessingSpecification,
    pub master_attempts: u8,
    pub master_wait_time: u8,
    pub segment_attempts: u8,
    pub segment_timeout: u8,
}

impl Default for MasterInterceptArguments {
    fn default() -> Self {
        Self {
            preprocessing: ProcessingSpecification::default(),
            master_attempts: 5,
            master_wait_time: 6,
            segment_attempts: 3,
            segment_timeout: 5,
        }
    }
}

/////////////////////////////////////////////////////
// DownloadStatus
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize)]
pub enum DownloadStatus {
    Starting,

    FindingIndex { attempt: u8 },
    DownloadingIndex,
    ParsingIndex,

    FindingMaster { attempt: u8 },
    DownloadingMaster,
    ParsingMaster,
    DownloadingPlaylist,
    ParsingPlaylist,

    Downloading { segment: u32, total_segments: u32 },

    Complete,
    Failed { message: String },
}

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Failed to retrieve domain from \"{0}\"")]
    FailedToRetrieveDomainFromURL(Url),

    #[error("{0}")]
    IndexInterceptError(index::IndexInterceptError),
    #[error("{0}")]
    MasterInterceptError(master::MasterInterceptError),

    #[error("Failed to start downloading data from \"{url}\" with error: {error}")]
    FailedToStart { url: Url, error: String },
    #[error("Failed to open output file \"{file}\" with error: {error}", file = file.display())]
    FailedToOpenOutputFile { file: PathBuf, error: String },
    #[error("Request to \"{url}\" failed with exit code: {exit_code}")]
    RequestFailed { url: Url, exit_code: i32 },
    #[error("Failed to write bytes to \"{file}\" due to error: {error}", file = file.display())]
    FailedToWriteBytes { file: PathBuf, error: String },
}

pub async fn download_file(
    specification: &DownloadSpecification, flaresolverr_url: &Url, status: Arc<RwLock<DownloadStatus>>, input_url: &Url, output_file: &Path,
    uses_cloudflare: bool,
) -> Result<(), DownloadError> {
    let base_url = {
        let scheme = input_url.scheme();
        let host = input_url
            .host_str()
            .ok_or(DownloadError::FailedToRetrieveDomainFromURL(input_url.clone()))?;

        Url::parse(&format!("{}://{}", scheme, host)).map_err(|_error| DownloadError::FailedToRetrieveDomainFromURL(input_url.clone()))?
    };

    // TODO: Only get this if cloudflare
    let credentials = request::get_credentials(flaresolverr_url, &base_url).await.unwrap();

    *status.write().unwrap() = DownloadStatus::Starting;
    match &specification.method {
        DownloadMethod::IndexInterception(index_specification) => {
            let arguments = IndexInterceptArguments {
                preprocessing: specification.preprocessing.clone(),
                index_attempts: index_specification.retries,
                index_wait_time: index_specification.wait_time,
                ..IndexInterceptArguments::default()
            };

            let index_data = index::IndexData::get_from(input_url, &arguments, &credentials, Arc::clone(&status))
                .await
                .map_err(DownloadError::IndexInterceptError)?;

            index::download_file(&index_data, &arguments, &credentials, output_file, status).await
        },
        DownloadMethod::MasterInterception(master_specification) => {
            let arguments = MasterInterceptArguments {
                preprocessing: specification.preprocessing.clone(),
                master_attempts: master_specification.retries,
                master_wait_time: master_specification.wait_time,
                ..MasterInterceptArguments::default()
            };

            let master_data = master::MasterData::get_from(input_url, &arguments, &credentials, Arc::clone(&status))
                .await
                .map_err(DownloadError::MasterInterceptError)?;
            let playlist_data = master_data.resolutions.iter().next().unwrap().1;

            master::download_file(playlist_data, &arguments, &credentials, output_file, status).await
        },
        DownloadMethod::MP4Interception(_mp4_specification) => {
            // TODO: ...
            Ok(())
        },
    }
}

use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use thiserror::Error;
use url::Url;
use serde::Serialize;

mod index_intercept;
use index_intercept as index;
// mod mp4_intercept;
// use mp4_intercept as mp4;

use super::request;
use super::config::DownloadMethod;

/////////////////////////////////////////////////////
// DownloadArguments
/////////////////////////////////////////////////////
pub struct IndexInterceptArguments {
    pub index_attempts: u8,
    pub index_wait_time: u8,
    pub segment_attempts: u8,
    pub segment_timeout: u8,
}

impl Default for IndexInterceptArguments {
    fn default() -> Self {
        Self {
            index_attempts: 5,
            index_wait_time: 6,
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
    method: &DownloadMethod, flaresolverr_url: &Url, status: Arc<RwLock<DownloadStatus>>, input_url: &Url, output_file: &Path, uses_cloudflare: bool,
) -> Result<(), DownloadError> {
    let referer = {
        let scheme = input_url.scheme();
        let host = input_url
            .host_str()
            .ok_or(DownloadError::FailedToRetrieveDomainFromURL(input_url.clone()))?;

        Url::parse(&format!("{}://{}", scheme, host)).map_err(|_error| DownloadError::FailedToRetrieveDomainFromURL(input_url.clone()))?
    };

    // TODO: Only get this if cloudflare
    let credentials = request::get_credentials(flaresolverr_url, &referer).await.unwrap();

    *status.write().unwrap() = DownloadStatus::Starting;
    match method {
        DownloadMethod::IndexInterception(specification) => {
            let arguments = IndexInterceptArguments {
                index_attempts: specification.retries,
                index_wait_time: specification.wait_time,
                ..IndexInterceptArguments::default()
            };

            let index_data = index::IndexData::get_from(&input_url, &arguments, &credentials, Arc::clone(&status))
                .await
                .map_err(DownloadError::IndexInterceptError)?;

            index::download_file(index_data, &arguments, &credentials, output_file, status).await
        },
        DownloadMethod::MP4Interception(_specification) => {
            // TODO: ...
            Ok(())
        },
    }
}

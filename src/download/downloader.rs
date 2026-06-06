use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;
use url::Url;
use tokio::fs::OpenOptions;

use super::super::config;
use super::super::request;
use super::super::streams;

use super::m3u;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Indexer download type doesn't match stream type passed in")]
    InvalidStreamIndexerCombo,
    #[error("Failed to retrieve domain from \"{0}\"")]
    FailedToRetrieveDomainFromURL(Url),

    #[error("Failed to start downloading data from \"{url}\" with error: {error}")]
    FailedToStart { url: Url, error: String },
    #[error("Failed to open output file \"{file}\" with error: {error}", file = file.display())]
    FailedToOpenOutputFile { file: PathBuf, error: String },
    #[error("Request error: {0}")]
    RequestFailed(request::RequestError),
    #[error("Failed to write bytes to disk due to error: {0}")]
    FailedToWriteBytes(String),
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub async fn download_stream(
    indexer: &config::Indexer, stream: streams::Stream, requester: &request::Requester, output_file: &Path,
) -> Result<(), DownloadError> {
    // Check of indexer's download_method against stream's stream_type
    if (matches!(indexer.download.method, config::DownloadMethod::IndexInterception(_)) && !matches!(stream.stream_type, streams::StreamType::M3U(_)))
        || (matches!(indexer.download.method, config::DownloadMethod::MasterInterception(_))
            && !matches!(stream.stream_type, streams::StreamType::M3U(_)))
    {
        return Err(DownloadError::InvalidStreamIndexerCombo);
    }

    // Open output
    trace!("Opening file \"{}\" for writing...", output_file.display());

    let mut file = OpenOptions::new().create(true).append(true).open(output_file).await.map_err(|error| {
        trace!("Failed to open \"{}\", error: {:?}, source: {:?}", output_file.display(), error, error.source());

        DownloadError::FailedToOpenOutputFile {
            file: output_file.to_path_buf(),
            error: error.to_string(),
        }
    })?;

    trace!("File \"{}\" successfully opened.", output_file.display());

    // Download based of of stream_type
    match stream.stream_type {
        streams::StreamType::M3U(segments) => {
            let (segment_attempts, segment_timeout) = {
                match &indexer.download.method {
                    // TODO: Replace these with environment somehow
                    config::DownloadMethod::IndexInterception(specification) => (specification.retries, specification.wait_time),
                    config::DownloadMethod::MasterInterception(specification) => (specification.retries, specification.wait_time),
                    _ => panic!("Internal logic error, unable to reach this path."),
                }
            };

            m3u::download_segments(
                indexer,
                segments,
                m3u::SegmentDownloadArguments {
                    segment_preprocessing: indexer.download.segment_pre_download.clone(),
                    segment_postprocessing: indexer.download.segment_post_download.clone(),
                    segment_timeout: segment_timeout,
                    segment_retries: segment_attempts,
                },
                requester,
                &mut file,
            )
            .await
        },
    }
}

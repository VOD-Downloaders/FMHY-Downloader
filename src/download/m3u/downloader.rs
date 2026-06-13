use url::Url;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::download::DownloadError;
use crate::config;
use crate::request;

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub async fn download_segments(
    indexer: &config::Indexer, segments: Vec<Url>, requester: &request::Requester, output_file: &mut File,
) -> Result<(), DownloadError> {
    trace!("Starting segments download...");

    for segment in segments {
        trace!("Downloading segment from: \"{}\"...", segment);

        let mut last_error: Option<DownloadError> = None;

        for attempt in 1..=indexer.download.segment_download.segment_attempts {
            match download_segment(indexer, &segment, requester, output_file).await {
                Ok(_) => {
                    last_error = None;
                    break;
                },
                Err(error) => {
                    warning!(
                        "[Attempt {}/{}] For segment \"{}\" failed with error: {}.",
                        attempt,
                        indexer.download.segment_download.segment_attempts,
                        segment.as_str(),
                        error
                    );

                    last_error = Some(error);
                },
            }
        }

        if let Some(error) = last_error {
            return Err(error);
        }
    }

    Ok(())
}

async fn download_segment(indexer: &config::Indexer, url: &Url, requester: &request::Requester, output_file: &mut File) -> Result<(), DownloadError> {
    let contents = requester
        .get_file_contents(url, Some(indexer.download.segment_download.headers.clone()))
        .await
        .map_err(DownloadError::RequestFailed)?;

    if contents.len()
        <= (indexer.download.segment_post_download.remove_front_bytes + indexer.download.segment_post_download.remove_back_bytes) as usize
    {
        return Err(DownloadError::FailedToWriteBytes(
            "Downloaded amount of bytes is less than the amount specified in postprocessing arguments.".to_string(),
        ));
    }

    let clean_bytes = &contents[indexer.download.segment_post_download.remove_front_bytes as usize
        ..(contents.len() - indexer.download.segment_post_download.remove_back_bytes as usize)];

    output_file
        .write_all(clean_bytes)
        .await
        .map_err(|error| DownloadError::FailedToWriteBytes(error.to_string()))?;

    Ok(())
}

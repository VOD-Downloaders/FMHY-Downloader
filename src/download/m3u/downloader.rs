use tokio::io::AsyncWriteExt;
use url::Url;
use tokio::fs::File;

use super::super::DownloadError;
use super::super::super::config;
use super::super::super::request;

/////////////////////////////////////////////////////
// SegmentDownloadArguments
/////////////////////////////////////////////////////
pub struct SegmentDownloadArguments {
    pub segment_preprocessing: config::PreDownloadSpecifiation,
    pub segment_postprocessing: config::PostDownloadSpecification,
    pub segment_retries: u8,
    pub segment_timeout: u8,
}

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub async fn download_segments(
    indexer: &config::Indexer, segments: Vec<Url>, arguments: SegmentDownloadArguments, requester: &request::Requester, output_file: &mut File,
) -> Result<(), DownloadError> {
    trace!("Starting segments download...");

    for segment in segments {
        trace!("Downloading segment from: \"{}\"...", segment);

        let mut last_error: Option<DownloadError> = None;

        for attempt in 1..=arguments.segment_retries {
            match download_segment(indexer, &segment, &arguments, requester, output_file).await {
                Ok(_) => {
                    last_error = None;
                    break;
                },
                Err(error) => {
                    warning!(
                        "[Attempt {}/{}] For segment \"{}\" failed with error: {}.",
                        attempt,
                        arguments.segment_retries,
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

async fn download_segment(
    indexer: &config::Indexer, url: &Url, arguments: &SegmentDownloadArguments, requester: &request::Requester, output_file: &mut File,
) -> Result<(), DownloadError> {
    let mut preprocessing = arguments.segment_preprocessing.clone();
    preprocessing.resolve_variables(&indexer.url, url);

    let contents = requester
        .get_file_contents(url, Some(preprocessing.headers.clone()))
        .await
        .map_err(DownloadError::RequestFailed)?;

    if contents.len() <= (arguments.segment_postprocessing.remove_front_bytes + arguments.segment_postprocessing.remove_back_bytes) as usize {
        return Err(DownloadError::FailedToWriteBytes(
            "Downloaded amount of bytes is less than the amount specified in postprocessing arguments.".to_string(),
        ));
    }

    let clean_bytes = &contents[arguments.segment_postprocessing.remove_front_bytes as usize
        ..(contents.len() - arguments.segment_postprocessing.remove_back_bytes as usize)];

    output_file
        .write_all(clean_bytes)
        .await
        .map_err(|error| DownloadError::FailedToWriteBytes(error.to_string()))?;

    Ok(())
}

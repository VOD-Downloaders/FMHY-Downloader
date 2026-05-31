use std::error::Error;
use std::sync::{Arc, RwLock};
use std::path::Path;

use url::Url;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::IndexData;
use super::super::DownloadError;
use super::super::DownloadStatus;
use super::super::IndexInterceptArguments;
use super::super::super::request;

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub async fn download_file(
    data: &IndexData, arguments: &IndexInterceptArguments, credentials: &request::Credentials, output_file: &Path,
    status: Arc<RwLock<DownloadStatus>>,
) -> Result<(), DownloadError> {
    trace!("Opening file \"{}\" for writing...", output_file.display());

    let mut file = OpenOptions::new().create(true).append(true).open(output_file).await.map_err(|error| {
        trace!("Failed to open \"{}\", error: {:?}, source: {:?}", output_file.display(), error, error.source());

        DownloadError::FailedToOpenOutputFile {
            file: output_file.to_path_buf(),
            error: error.to_string(),
        }
    })?;

    trace!("File \"{}\" successfully opened.", output_file.display());

    info!("Downloading to \"{}\"...", output_file.display());
    *status.write().unwrap() = DownloadStatus::Downloading {
        segment: 1,
        total_segments: data.files.len() as u32,
    };

    for (i, segment) in data.files.iter().enumerate() {
        *status.write().unwrap() = DownloadStatus::Downloading {
            segment: i as u32,
            total_segments: data.files.len() as u32,
        };

        let segment_url = Url::parse(segment.as_str());
        let full_url = {
            match segment_url {
                Ok(url) => url,
                Err(_error) => data.base_url.join(segment.as_str()).unwrap(),
            }
        };

        let mut last_error: Option<DownloadError> = None;

        for attempt in 1..=arguments.segment_attempts {
            match download_segment(&full_url, arguments, credentials, &data.referer, &mut file, output_file).await {
                Ok(_) => {
                    break;
                },
                Err(error) => {
                    warning!(
                        "[Attempt {}/{}] For segment \"{}\" failed with error: {}.",
                        attempt,
                        arguments.segment_attempts,
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

    *status.write().unwrap() = DownloadStatus::Complete;
    Ok(())
}

async fn download_segment(
    url: &Url, arguments: &IndexInterceptArguments, credentials: &request::Credentials, referer: &Url, output_file: &mut File, file_path: &Path,
) -> Result<(), DownloadError> {
    let referer_header = String::from("Referer: ") + referer.as_str();
    let user_agent_header = String::from("User-Agent: ") + credentials.user_agent.as_str();
    let connect_timeout = arguments.segment_timeout.to_string();
    let max_timeout = arguments.segment_timeout.to_string();

    trace!("Sending GET request to \"{}\", with headers: [\"{}\", \"{}\"].", url, referer_header, user_agent_header);

    let output = Command::new("curl")
        .args([
            "--silent",
            "--fail",
            "--connect-timeout",
            connect_timeout.as_str(),
            "--max-time",
            max_timeout.as_str(),
            "-H",
            referer_header.as_str(),
            "-H",
            user_agent_header.as_str(),
            "--output",
            "-", // write to stdout
            url.as_str(),
        ])
        .output()
        .await
        .map_err(|error| DownloadError::FailedToStart {
            url: url.clone(),
            error: error.to_string(),
        })?;

    trace!("GET request exited with status: {}, output: {}", output.status, String::from_utf8_lossy(&output.stderr));

    if output.stdout.len() <= arguments.preprocessing.remove_bytes as usize {
        return Err(DownloadError::FailedToWriteBytes {
            file: file_path.to_path_buf(),
            error: "Downloaded amount of bytes is less than the amount to remove due to preprocessing arguments.".to_string(),
        });
    }

    let clean_bytes = &output.stdout[arguments.preprocessing.remove_bytes as usize..];

    output_file
        .write_all(clean_bytes)
        .await
        .map_err(|error| DownloadError::FailedToWriteBytes {
            file: file_path.to_path_buf(),
            error: error.to_string(),
        })?;

    Ok(())
}

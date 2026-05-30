use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;
use url::Url;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::IndexData;
use super::super::super::env;
use super::super::super::request;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Failed to start downloading data from \"{url}\" with error: {error}")]
    FailedToStart { url: String, error: String },
    #[error("Failed to open output file \"{file}\" with error: {error}", file = file.display())]
    FailedToOpenOutputFile { file: PathBuf, error: String },
    #[error("Request to \"{url}\" failed with exit code: {exit_code}")]
    RequestFailed { url: String, exit_code: i32 },
    #[error("Failed to write bytes to \"{file}\" due to error: {error}", file = file.display())]
    FailedToWriteBytes { file: PathBuf, error: String },
}

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub async fn download_file(
    environment: &env::EnvOptions, credentials: &request::Credentials, index: IndexData, output_file: &Path,
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

    for segment in index.files {
        let segment_url = Url::parse(segment.as_str());
        let full_url = {
            match segment_url {
                Ok(url) => url,
                Err(_error) => index.base_url.join(segment.as_str()).unwrap(),
            }
        };

        let mut last_error: Option<DownloadError> = None;

        for attempt in 1..=environment.segment_retry_attempts {
            match download_segment(environment, credentials, &full_url, &index.referer, &mut file, output_file).await {
                Ok(_) => {
                    break;
                },
                Err(error) => {
                    warning!(
                        "[Attempt {}/{}] For segment \"{}\" failed with error: {}.",
                        attempt,
                        environment.segment_retry_attempts,
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
    environment: &env::EnvOptions, credentials: &request::Credentials, url: &Url, referer: &Url, output_file: &mut File, file_path: &Path,
) -> Result<(), DownloadError> {
    let referer_header = String::from("Referer: ") + referer.as_str();
    let user_agent_header = String::from("User-Agent: ") + credentials.user_agent.as_str();
    let connect_timeout = environment.segment_download_timeout.to_string();
    let max_timeout = environment.segment_download_timeout.to_string();

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
            url: url.to_string(),
            error: error.to_string(),
        })?;

    trace!("GET request exited with status: {}, output: {}", output.status, String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(DownloadError::RequestFailed {
            url: url.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    output_file
        .write_all(&output.stdout)
        .await
        .map_err(|error| DownloadError::FailedToWriteBytes {
            file: file_path.to_path_buf(),
            error: error.to_string(),
        })?;

    Ok(())
}

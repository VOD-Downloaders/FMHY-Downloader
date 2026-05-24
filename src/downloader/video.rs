use core::fmt;

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use super::IndexData;
use super::super::env;
use super::super::request;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum DownloadError {
    FailedToStart { url: String, error: String },
    FailedToOpenOutputFile { file: PathBuf, error: String },
    RequestFailed { url: String, exit_code: i32 },
    FailedToWriteBytes { file: PathBuf, error: String },
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadError::FailedToStart { url, error } => {
                write!(f, "Failed to start downloading data from \"{}\" with error: {}", url, error)
            },
            DownloadError::FailedToOpenOutputFile { file, error } => {
                write!(f, "Failed to open output file \"{}\" with error: {}", file.display(), error)
            },
            DownloadError::RequestFailed { url, exit_code } => {
                write!(f, "Request to \"{}\" failed with exit code: {}", url, exit_code)
            },
            DownloadError::FailedToWriteBytes { file, error } => {
                write!(f, "Failed to write bytes to \"{}\" due to error: {}", file.display(), error)
            },
        }
    }
}

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub fn download_file(
    environment: &env::EnvOptions, credentials: &request::Credentials, index: IndexData, output_file: &Path,
) -> Result<(), DownloadError> {
    let mut file = OpenOptions::new().create(true).append(true).open(output_file).map_err(|error| {
        return DownloadError::FailedToOpenOutputFile {
            file: output_file.to_path_buf(),
            error: error.to_string(),
        };
    })?;

    for segment in index.files {
        let full_url = index.base_url.clone() + segment.as_str();

        let mut last_error: Option<DownloadError> = None;

        for attempt in 1..=environment.segment_retry_attempts {
            match download_segment(environment, credentials, full_url.as_str(), index.referer.as_str(), &mut file, output_file) {
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

fn download_segment(
    environment: &env::EnvOptions, credentials: &request::Credentials, url: &str, referer: &str, output_file: &mut File, file_path: &Path,
) -> Result<(), DownloadError> {
    let referer_header = String::from("Referer: ") + referer;
    let user_agent_header = String::from("User-Agent: ") + credentials.user_agent.as_str();
    let connect_timeout = environment.segment_download_timeout.to_string();
    let max_timeout = environment.segment_download_timeout.to_string();

    trace!("Sending GET request to \"{}\".", url);

    let output = std::process::Command::new("curl")
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
            url,
        ])
        .output()
        .map_err(|error| DownloadError::FailedToStart {
            url: url.to_string(),
            error: error.to_string(),
        })?;

    trace!("GET request exited with status: {}", output.status);
    trace!("GET request output: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(DownloadError::RequestFailed {
            url: url.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    output_file.write_all(&output.stdout).map_err(|error| DownloadError::FailedToWriteBytes {
        file: file_path.to_path_buf(),
        error: error.to_string(),
    })?;

    Ok(())
}

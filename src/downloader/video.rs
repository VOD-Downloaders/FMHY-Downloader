use core::fmt;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use reqwest::blocking::Client;

use super::IndexData;
use super::super::env;
use super::super::request;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum DownloadError {
    FailedToCreateClient { error: String },
    FailedToStart { url: String, error: String },
    FailedToOpenOutputFile { file: PathBuf, error: String },
    RequestFailed { url: String, exit_code: i32 },
    FailedToReadBytes { url: String, error: String },
    FailedToWriteBytes { file: PathBuf, error: String },
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadError::FailedToCreateClient { error } => {
                write!(f, "Failed to create HTTP client with error: {}", error)
            },
            DownloadError::FailedToStart { url, error } => {
                write!(f, "Failed to start downloading data from \"{}\" with error: {}", url, error)
            },
            DownloadError::FailedToOpenOutputFile { file, error } => {
                write!(f, "Failed to open output file \"{}\" with error: {}", file.display(), error)
            },
            DownloadError::RequestFailed { url, exit_code } => {
                write!(f, "Request to \"{}\" failed with exit code: {}", url, exit_code)
            },
            DownloadError::FailedToReadBytes { url, error } => {
                write!(f, "Failed to read the bytes from \"{}\"'s response, error: {}", url, error)
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
    environment: &env::EnvOptions, index: IndexData, credentials: &request::Credentials, output_file: &Path,
) -> Result<(), DownloadError> {
    let build_client = || -> Result<Client, DownloadError> {
        Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(environment.segment_download_timeout as u64))
            .user_agent(credentials.user_agent.clone())
            .referer(false)
            .no_proxy()
            .connection_verbose(true)
            .build()
            .map_err(|error| DownloadError::FailedToCreateClient { error: error.to_string() })
    };

    let mut client = build_client()?;
    let mut headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    headers.insert("Referer", index.referer.parse().unwrap());

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
            match download_segment(environment, full_url.as_str(), &mut client, headers.clone(), &mut file, output_file) {
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

                    if let DownloadError::FailedToStart { url: _, error: _ } = &error {
                        // Might be a timeout, recreate the client
                        // trace!("Segment failed with FailedToStart so recreating client...");
                        //trace!("Client recreated successfully");
                    }

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
    _environment: &env::EnvOptions, url: &str, client: &mut Client, headers: reqwest::header::HeaderMap, output_file: &mut File, file_path: &Path,
) -> Result<(), DownloadError> {
    trace!("Sending GET request to: \"{}\" with these headers: {:?}", url, headers);

    let response = client.get(url).headers(headers).send().map_err(|error| {
        trace!("Failed to send request, error: {}, error source: {:?}", error, error.source());

        return DownloadError::FailedToStart {
            url: url.to_string(),
            error: error.to_string(),
        };
    })?;

    trace!("Response status: {}", response.status());

    if !response.status().is_success() {
        return Err(DownloadError::RequestFailed {
            url: url.to_string(),
            exit_code: response.status().as_u16() as i32,
        });
    }

    match response.bytes() {
        Ok(bytes) => {
            trace!("Received {} bytes, writing to: {}", bytes.len(), file_path.display());
            output_file.write_all(&bytes).map_err(|error| {
                return DownloadError::FailedToWriteBytes {
                    file: file_path.to_path_buf(),
                    error: error.to_string(),
                };
            })?;
            Ok(())
        },
        Err(error) => Err(DownloadError::FailedToReadBytes {
            url: url.to_string(),
            error: error.to_string(),
        }),
    }
}

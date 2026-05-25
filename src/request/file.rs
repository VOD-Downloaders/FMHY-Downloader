use core::fmt;

use reqwest::blocking::Client;

use super::Credentials;

/////////////////////////////////////////////////////
// RequestFileError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum RequestFileError {
    FailedToStart { url: String, error: String },
    RequestFailed { url: String, exit_code: i32 },
    FailedToReadBytes { url: String, error: String },
}

impl fmt::Display for RequestFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RequestFileError::FailedToStart { url, error } => {
                write!(f, "Failed to start downloading data from \"{}\" with error: {}", url, error)
            },
            RequestFileError::RequestFailed { url, exit_code } => {
                write!(f, "Request to \"{}\" failed with exit code: {}", url, exit_code)
            },
            RequestFileError::FailedToReadBytes { url, error } => {
                write!(f, "Failed to read the bytes from \"{}\"'s response, error: {}", url, error)
            },
        }
    }
}

/////////////////////////////////////////////////////
// File
/////////////////////////////////////////////////////
pub fn get_file_contents(url: &str, credentials: &Credentials, referer: &str) -> Result<Vec<u8>, RequestFileError> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .connect_timeout(std::time::Duration::from_secs(30))
        .user_agent(credentials.user_agent.clone())
        .referer(false)
        .build()
        .map_err(|error| RequestFileError::FailedToStart {
            url: url.to_string(),
            error: error.to_string(),
        })?;

    let mut headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    headers.insert("Referer", referer.parse().unwrap());

    let response = client.get(url).headers(headers).send().map_err(|error| RequestFileError::FailedToStart {
        url: url.to_string(),
        error: error.to_string(),
    })?;

    if !response.status().is_success() {
        return Err(RequestFileError::RequestFailed {
            url: url.to_string(),
            exit_code: response.status().as_u16() as i32,
        });
    }

    match response.bytes() {
        Ok(bytes) => {
            let bytes: Vec<u8> = bytes.into();
            Ok(bytes)
        },
        Err(error) => Err(RequestFileError::FailedToReadBytes {
            url: url.to_string(),
            error: error.to_string(),
        }),
    }
}

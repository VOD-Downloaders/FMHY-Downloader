use url::Url;

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

use tokio::process::Command;

/////////////////////////////////////////////////////
// CurlRequester
/////////////////////////////////////////////////////
pub struct CurlRequester {
    pub specification: RequesterSpecification,
}

impl CurlRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        Ok(Self {
            specification: specification,
        })
    }

    pub async fn get_file_contents(&self, url: &Url, headers: Option<HeaderMap>) -> Result<Vec<u8>, RequestError> {
        let mut final_headers: reqwest::header::HeaderMap = self.specification.headers.clone();
        for (key, val) in &headers.unwrap_or_default() {
            final_headers.insert(key, val.clone());
        }

        let connect_timeout_str = self.specification.connect_timeout.to_string();
        let max_timeout_str = self.specification.max_timeout.to_string();

        let headers: Vec<String> = final_headers
            .iter()
            .flat_map(|(key, value)| ["-H".to_string(), format!("{}: {}", key, value.to_str().unwrap_or_default())])
            .collect();

        let output = Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--connect-timeout",
                connect_timeout_str.as_str(),
                "--max-time",
                max_timeout_str.as_str(),
            ])
            .args(&headers)
            .args([
                "--output",
                "-", // Write to stdout
                url.as_str(),
            ])
            .output()
            .await
            .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?;

        if !output.status.success() {
            return Err(RequestError::RequestFailed(format!("Exit status: {}, Output: {}", output.status, String::from_utf8_lossy(&output.stderr))));
        }

        Ok(output.stdout)
    }
}

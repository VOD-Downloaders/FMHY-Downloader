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

    pub async fn get_file_contents(&self, url: &Url, headers: HeaderMap) -> Result<Vec<u8>, RequestError> {
        let connect_timeout_str = self.specification.connect_timeout.to_string();
        let max_timeout_str = self.specification.max_timeout.to_string();

        let mut headers: Vec<String> = headers
            .iter()
            .flat_map(|(key, value)| ["-H".to_string(), format!("{}: {}", key, value.to_str().unwrap_or_default())])
            .collect();

        headers.push("-H".to_string());
        headers.push(String::from("User-Agent: ") + self.specification.user_agent.as_str());

        let mut command_base = Command::new("curl");
        let command = command_base
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
            ]);

        trace!("Full get_file_contents command is: {:?}", command);

        let output = command
            .output()
            .await
            .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?;

        if !output.status.success() {
            return Err(RequestError::RequestFailed(format!("Exit status: {}, Output: {}", output.status, String::from_utf8_lossy(&output.stderr))));
        }

        Ok(output.stdout)
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        return &self.specification;
    }
}

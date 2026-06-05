use url::Url;

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// NativeRequester
/////////////////////////////////////////////////////
pub struct NativeRequester {
    pub specification: RequesterSpecification,
    pub client: reqwest::blocking::Client,
}

impl NativeRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        const MAX_REDIRECTS: usize = 10;

        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .timeout(std::time::Duration::from_secs(specification.max_timeout))
            .connect_timeout(std::time::Duration::from_secs(specification.connect_timeout))
            .user_agent(specification.user_agent.as_str())
            .referer(false)
            .build()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?;

        Ok(Self {
            specification: specification,
            client: client,
        })
    }

    pub async fn get_file_contents(&self, url: &Url, headers: HeaderMap) -> Result<Vec<u8>, RequestError> {
        let response = self
            .client
            .get(url.as_str())
            .headers(headers)
            .send()
            .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?;

        if !response.status().is_success() {
            return Err(RequestError::RequestFailed(format!("Status: {}, Error: {}", response.status(), response.error_for_status().unwrap_err())));
        }

        match response.bytes() {
            Ok(bytes) => Ok(bytes.into()),
            Err(error) => Err(RequestError::FailedToReadBytes(error.to_string())),
        }
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        return &self.specification;
    }
}

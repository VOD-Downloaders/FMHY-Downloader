use thiserror::Error;
use url::Url;
use rand::prelude::*;

use super::NativeRequester;
use super::CurlRequester;
use super::FlaresolveddRequester;

/////////////////////////////////////////////////////
// RequestError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to create requester, error: {0}")]
    FailedToCreate(String),
    #[error("Failed to send request, error: {0}")]
    RequestFailedToSend(String),
    #[error("Request failed with error: {0}")]
    RequestFailed(String),
    #[error("Failed to read response's bytes with error: {0}")]
    FailedToReadBytes(String),
}

/////////////////////////////////////////////////////
// RequesterSpecification
/////////////////////////////////////////////////////
pub type HeaderMap = reqwest::header::HeaderMap;

pub struct RequesterSpecification {
    pub user_agent: String,
    pub headers: HeaderMap,
    pub connect_timeout: u64,
    pub max_timeout: u64,
}

impl Default for RequesterSpecification {
    fn default() -> Self {
        Self {
            user_agent: get_random_user_agent().to_string(),
            headers: HeaderMap::new(),
            connect_timeout: 10,
            max_timeout: 10,
        }
    }
}

/////////////////////////////////////////////////////
// Requester
/////////////////////////////////////////////////////
pub enum Requester {
    Native(NativeRequester),
    Curl(CurlRequester),
    Flaresolvedd(FlaresolveddRequester),
}

impl Requester {
    pub fn get_native(specification: RequesterSpecification) -> Result<Self, RequestError> {
        Ok(Requester::Native(NativeRequester::new(specification)?))
    }

    pub fn get_curl(specification: RequesterSpecification) -> Result<Self, RequestError> {
        Ok(Requester::Curl(CurlRequester::new(specification)?))
    }

    // NOTE: user_agent in specification will be ignored and replaced by flaresolverr's response.
    pub fn get_flaresolvedd(specification: RequesterSpecification, begin_url: &Url) -> Result<Self, RequestError> {
        Ok(Requester::Flaresolvedd(FlaresolveddRequester::new(specification, begin_url)?))
    }

    pub async fn get_file_contents(&self, url: &Url, headers: Option<HeaderMap>) -> Result<Vec<u8>, RequestError> {
        match self {
            Requester::Native(instance) => instance.get_file_contents(url, headers).await,
            Requester::Curl(instance) => instance.get_file_contents(url, headers).await,
            Requester::Flaresolvedd(instance) => instance.get_file_contents(url, headers).await,
        }
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        match self {
            Requester::Native(instance) => instance.get_specification(),
            Requester::Curl(instance) => instance.get_specification(),
            Requester::Flaresolvedd(instance) => instance.get_specification(),
        }
    }
}

/////////////////////////////////////////////////////
// Helper methods
/////////////////////////////////////////////////////
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2.1 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
];

pub fn get_random_user_agent() -> &'static str {
    let mut rng = rand::rng();
    USER_AGENTS.choose(&mut rng).copied().unwrap()
}

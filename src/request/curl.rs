use url::Url;

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// CurlRequester
/////////////////////////////////////////////////////
pub struct CurlRequester {}

impl CurlRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        Ok(Self {})
    }

    pub fn get_file_contents(&self, url: &Url, headers: Option<HeaderMap>) -> Result<Vec<u8>, RequestError> {
        todo!()
    }
}

use url::Url;

use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// FlaresolveddRequester
/////////////////////////////////////////////////////
pub struct FlaresolveddRequester {}

impl FlaresolveddRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        Ok(Self {})
    }

    pub fn get_file_contents(&self, url: &Url) -> Result<Vec<u8>, RequestError> {
        todo!()
    }
}

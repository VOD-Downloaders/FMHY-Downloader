use url::Url;

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// FlaresolveddRequester
/////////////////////////////////////////////////////
pub struct FlaresolveddRequester {
    pub specification: RequesterSpecification,
    // TODO: Session stuff
}

impl FlaresolveddRequester {
    pub fn new(specification: RequesterSpecification, begin_url: &Url) -> Result<Self, RequestError> {
        // TODO: Create flaresolverr session
        todo!()
    }

    pub async fn get_file_contents(&self, url: &Url, headers: Option<HeaderMap>) -> Result<Vec<u8>, RequestError> {
        // TODO: Send request through flaresolverr session
        todo!()
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        return &self.specification;
    }
}

impl Drop for FlaresolveddRequester {
    fn drop(&mut self) {
        // TODO: Destroy flaresolverr session
        todo!()
    }
}

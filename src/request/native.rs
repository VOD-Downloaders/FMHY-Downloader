use url::Url;

use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// NativeRequester
/////////////////////////////////////////////////////
pub struct NativeRequester {
    specification: RequesterSpecification,
    client: reqwest::blocking::Client,
}

impl NativeRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        const MAX_REDIRECTS: usize = 10;

        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
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

    pub fn get_file_contents(&self, url: &Url) -> Result<Vec<u8>, RequestError> {
        todo!()
    }
}

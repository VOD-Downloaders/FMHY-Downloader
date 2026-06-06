use std::collections::HashMap;
use std::ops::Deref;

use url::Url;
use serde::{Serialize, Deserialize};

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// Bodies
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct FlareSolverrCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct FlareSolverrSolution {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub response: String,
    pub cookies: Vec<FlareSolverrCookie>,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
}

#[derive(Debug, Deserialize)]
pub struct FlareSolverrResponse {
    pub status: String,
    pub message: String,
    pub solution: Option<FlareSolverrSolution>,
    pub session: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FlareSolverrCommand {
    pub cmd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/////////////////////////////////////////////////////
// FlaresolveddRequester
/////////////////////////////////////////////////////
pub struct FlaresolveddRequester {
    pub specification: RequesterSpecification,

    pub client: reqwest::blocking::Client,
    pub flaresolverr_url: Url,
    pub session_id: Option<String>,
}

impl FlaresolveddRequester {
    pub fn new_native(mut specification: RequesterSpecification, flaresolverr_url: &Url, begin_url: &Url) -> Result<Self, RequestError> {
        const MAX_REDIRECTS: usize = 10;

        // Create native client
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .timeout(std::time::Duration::from_secs(specification.max_timeout))
            .connect_timeout(std::time::Duration::from_secs(specification.connect_timeout))
            .user_agent(specification.user_agent.as_str())
            .referer(false)
            .build()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?;

        // Visit initial URL
        let visit_initial_command = FlareSolverrCommand {
            cmd: "request.get".to_string(),
            session: None,
            url: Some(begin_url.to_string()),
        };

        trace!("Visiting initial URL \"{}\" with flaresolverr, command {:?}", begin_url, visit_initial_command);

        let solve_response: FlareSolverrResponse = client
            .post(flaresolverr_url.as_str())
            .json(&visit_initial_command)
            .send()
            .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?
            .json()
            .map_err(|error| RequestError::RequestFailed(error.to_string()))?;

        trace!("Got flaresolverr response from initial url: {:?}", solve_response);

        // Overwrite User-Agent load cookies
        if let Some(solution) = solve_response.solution {
            specification.user_agent = solution.user_agent;

            for cookie in solution.cookies {
                let name = reqwest::header::HeaderName::from_lowercase(cookie.name.to_lowercase().as_bytes());
                let value = reqwest::header::HeaderValue::from_str(cookie.value.as_str());

                if name.is_err() || value.is_err() {
                    trace!("Unable to convert this cookie {:?} to HTTP header.", cookie);
                    continue;
                }

                specification.headers.insert(name.unwrap(), value.unwrap());
            }
        }

        Ok(Self {
            specification: specification,
            client: client,
            flaresolverr_url: flaresolverr_url.clone(),
            session_id: None,
        })
    }

    pub fn new_session(mut specification: RequesterSpecification, flaresolverr_url: &Url) -> Result<Self, RequestError> {
        const MAX_REDIRECTS: usize = 10;

        // Create native client
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .timeout(std::time::Duration::from_secs(specification.max_timeout))
            .connect_timeout(std::time::Duration::from_secs(specification.connect_timeout))
            .user_agent(specification.user_agent.as_str())
            .referer(false)
            .build()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?;

        // Create flaresolverr session
        let create_session_command = FlareSolverrCommand {
            cmd: "sessions.create".to_string(),
            session: None,
            url: None,
        };

        trace!("Creating new flaresolverr session with command: {:?}", create_session_command);

        let create_session_response: FlareSolverrResponse = client
            .post(flaresolverr_url.as_str())
            .json(&create_session_command)
            .send()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?
            .json()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?;

        trace!("Got session response: {:?}", create_session_response);

        let session_id = create_session_response
            .session
            .ok_or(RequestError::FailedToCreate(format!("No session created: {}", create_session_response.message)))?;

        Ok(Self {
            specification: specification,
            client: client,
            flaresolverr_url: flaresolverr_url.clone(),
            session_id: Some(session_id),
        })
    }

    pub async fn get_file_contents(&self, url: &Url, headers: HeaderMap) -> Result<Vec<u8>, RequestError> {
        // SESSION
        if self.session_id.is_some() {
            let request_command = FlareSolverrCommand {
                cmd: "request.get".to_string(),
                session: self.session_id.clone(),
                url: Some(url.to_string()),
            };

            let response: FlareSolverrResponse = self
                .client
                .post(self.flaresolverr_url.as_str())
                .json(&request_command)
                .send()
                .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?
                .json()
                .map_err(|error| RequestError::RequestFailed(error.to_string()))?;

            if let Some(solution) = response.solution {
                Ok(solution.response.into_bytes())
            } else {
                Err(RequestError::RequestFailed("Failed to fetch via FlareSolverr proxy layer".to_string()))
            }
        }
        // COOKIES/HEADERS
        else {
            let response = self
                .client
                .get(url.as_str())
                .headers(headers.0)
                .send()
                .map_err(|error| RequestError::RequestFailedToSend(error.to_string()))?;

            if !response.status().is_success() {
                return Err(RequestError::RequestFailed(format!(
                    "Status: {}, Error: {}",
                    response.status(),
                    response.error_for_status().unwrap_err()
                )));
            }

            match response.bytes() {
                Ok(bytes) => Ok(bytes.into()),
                Err(error) => Err(RequestError::FailedToReadBytes(error.to_string())),
            }
        }
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        return &self.specification;
    }
}

impl Drop for FlaresolveddRequester {
    fn drop(&mut self) {
        if self.session_id.is_some() {
            let destroy_command = FlareSolverrCommand {
                cmd: "sessions.destroy".to_string(),
                session: self.session_id.clone(),
                url: None,
            };

            let _ = self.client.post(self.flaresolverr_url.as_str()).json(&destroy_command).send();
        }
    }
}

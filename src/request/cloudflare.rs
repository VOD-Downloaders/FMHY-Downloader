use core::fmt;

use serde::Deserialize;

/////////////////////////////////////////////////////
// RequestError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum RequestCredentialsError {
    FailedToPOSTFlaresolverr { error: String },
    FailedToGetBodyFromRequest,
    FailedToParseBody,
}

impl fmt::Display for RequestCredentialsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RequestCredentialsError::FailedToPOSTFlaresolverr { error } => {
                write!(f, "Failed to send request to flaresolverr with error: {}.", error)
            },
            RequestCredentialsError::FailedToGetBodyFromRequest => {
                write!(f, "Failed to access the body from flaresolverr request.")
            },
            RequestCredentialsError::FailedToParseBody => {
                write!(f, "Failed to parse the body from the flaresolverr request.")
            },
        }
    }
}

/////////////////////////////////////////////////////
// Responses
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub url: String,
    pub status: u32,
    pub cookies: Vec<Cookie>,

    #[serde(rename = "userAgent")]
    pub user_agent: String,
    // response: String,
}

#[derive(Debug, Deserialize)]
struct FlareSolverrResponse {
    // status: String,
    solution: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}

/////////////////////////////////////////////////////
// Request
/////////////////////////////////////////////////////
pub async fn get_credentials(flaresolverr_url: &str, url: &str) -> Result<Credentials, RequestCredentialsError> {
    trace!("Attempting to get credentials for \"{}\"...", url);

    let body = serde_json::json!({
        "cmd": "request.get",
        "url": url,
        "maxTimeout": 60000
    });

    let client = reqwest::Client::new();
    let response = client
        .post(flaresolverr_url)
        .json(&body)
        .send()
        .await
        .map_err(|error| RequestCredentialsError::FailedToPOSTFlaresolverr { error: error.to_string() })?;

    let status = response.status();

    trace!("Request to \"{}\" exited with status: {}", url, status);

    let body = response
        .text()
        .await
        .map_err(|_error| RequestCredentialsError::FailedToGetBodyFromRequest)?;
    let parsed: FlareSolverrResponse = serde_json::from_str(&body).map_err(|_error| RequestCredentialsError::FailedToParseBody)?;

    Ok(parsed.solution)
}

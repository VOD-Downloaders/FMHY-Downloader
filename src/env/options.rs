use core::fmt;
use std::env;

use super::super::logging::LogLevel;

/////////////////////////////////////////////////////
// EnvError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum EnvError {
    MissingFlaresolverrUrl,
    FlaresolverrUrlNoHTTP { url: String },
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EnvError::MissingFlaresolverrUrl => {
                write!(f, "FLARESOLVERR_URL is not set in the current environment.")
            },
            EnvError::FlaresolverrUrlNoHTTP { url } => {
                write!(f, "FLARESOLVERR_URL doesn't start with http or https, url: {}", url)
            },
        }
    }
}

/////////////////////////////////////////////////////
// Options
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct EnvOptions {
    log_level: LogLevel,
    flaresolverr_url: String,
}

impl Default for EnvOptions {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            flaresolverr_url: "".to_string(),
        }
    }
}

impl EnvOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_stdenv() -> Result<Self, EnvError> {
        let Ok(flaresolverr_url) = env::var("FLARESOLVERR_URL") else {
            return Err(EnvError::MissingFlaresolverrUrl);
        };

        Ok(Self {
            flaresolverr_url: flaresolverr_url,
            ..Default::default()
        })
    }

    fn parse_flaresolverr_url(url: &str) -> Result<String, EnvError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(EnvError::FlaresolverrUrlNoHTTP { url: url.to_string() });
        }

        Ok(url.to_string())
    }
}

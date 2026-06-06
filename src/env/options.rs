use std::env;

use thiserror::Error;
use url::Url;

use super::super::logging::LogLevel;

/////////////////////////////////////////////////////
// EnvError
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Error)]
pub enum EnvError {
    #[error("FLARESOLVERR_URL is set to an invalid url \"{url}\", error: {error}")]
    InvalidFlaresolverrUrl { url: String, error: url::ParseError },
    #[error("LOG_LEVEL contains invalid data (must be \"debug\", \"info\", \"warning\" or \"error\". Received: {log_level}")]
    InvalidLogLevel { log_level: String },
    #[error("Expected WEBUI_PORT to be a 16 bit unsigned integer, got: {port}")]
    InvalidWebUIPort { port: String },
}

/////////////////////////////////////////////////////
// Options
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct EnvOptions {
    pub log_level: LogLevel,
    pub flaresolverr_url: Option<Url>,
    pub webui_port: u16,
}

impl Default for EnvOptions {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            flaresolverr_url: None,
            webui_port: 8080,
        }
    }
}

impl EnvOptions {
    pub fn from_env() -> Result<Self, EnvError> {
        let default = EnvOptions::default();

        let log_level = Self::parse_log_level()?;
        let flaresolverr_url = Self::parse_flaresolverr_url()?;
        let webui_port = Self::parse_webui_port()?;

        Ok(Self {
            log_level: log_level.unwrap_or(default.log_level),
            flaresolverr_url: flaresolverr_url,
            webui_port: webui_port.unwrap_or(default.webui_port),
        })
    }

    fn parse_log_level() -> Result<Option<LogLevel>, EnvError> {
        let Ok(log_level) = env::var("LOG_LEVEL") else {
            return Ok(None);
        };

        match log_level.to_lowercase().as_str() {
            "debug" | "trace" => Ok(Some(LogLevel::Trace)),
            "info" => Ok(Some(LogLevel::Info)),
            "warning" => Ok(Some(LogLevel::Warn)),
            "error" => Ok(Some(LogLevel::Error)),
            _ => Err(EnvError::InvalidLogLevel { log_level: log_level }),
        }
    }

    fn parse_flaresolverr_url() -> Result<Option<Url>, EnvError> {
        let Ok(flaresolverr_url) = env::var("FLARESOLVERR_URL") else {
            return Ok(None);
        };

        let flaresolverr_url = Url::parse(flaresolverr_url.as_str()).map_err(|error| {
            return EnvError::InvalidFlaresolverrUrl {
                url: flaresolverr_url,
                error: error,
            };
        })?;

        Ok(Some(flaresolverr_url))
    }

    fn parse_webui_port() -> Result<Option<u16>, EnvError> {
        let Ok(port) = env::var("WEBUI_PORT") else {
            return Ok(None);
        };

        let port = port.parse::<u16>().map_err(|_error| EnvError::InvalidWebUIPort { port: port })?;

        Ok(Some(port))
    }
}

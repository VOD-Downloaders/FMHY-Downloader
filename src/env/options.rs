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
    InvalidLogLevel { log_level: String },
    InvalidWebUIPort { port: String },
    InvalidThreadCount { thread_count: String },
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
            EnvError::InvalidLogLevel { log_level } => {
                write!(f, "LOG_LEVEL contains invalid data (must be \"debug\", \"info\", \"warning\" or \"error\". Received: {}", log_level)
            },
            EnvError::InvalidWebUIPort { port } => {
                write!(f, "Expected the WebUI port to be a 16 bit unsigned integer, got: {}", port)
            },
            EnvError::InvalidThreadCount { thread_count } => {
                write!(f, "Expected the thread count to be an 8 bit unsigned integer, got: {}", thread_count)
            },
        }
    }
}

/////////////////////////////////////////////////////
// Options
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct EnvOptions {
    pub log_level: LogLevel,
    pub flaresolverr_url: String,
    pub webui_port: u16,
    pub download_threads: u8,
}

impl Default for EnvOptions {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            flaresolverr_url: "".to_string(),
            webui_port: 8080,
            download_threads: 1,
        }
    }
}

impl EnvOptions {
    pub fn from_env() -> Result<Self, EnvError> {
        let default = EnvOptions::default();

        let log_level = Self::parse_log_level()?;
        let flaresolverr_url = Self::parse_flaresolverr_url()?;
        let webui_port = Self::parse_webui_port()?;
        let download_threads = Self::parse_download_threads()?;

        Ok(Self {
            log_level: log_level.unwrap_or(default.log_level),
            flaresolverr_url: flaresolverr_url.to_string(),
            webui_port: webui_port.unwrap_or(default.webui_port),
            download_threads: download_threads.unwrap_or(default.download_threads),
            ..default
        })
    }

    fn parse_log_level() -> Result<Option<LogLevel>, EnvError> {
        let Ok(log_level) = env::var("LOG_LEVEL") else {
            return Ok(None);
        };

        match log_level.to_lowercase().as_str() {
            "debug" => Ok(Some(LogLevel::Trace)),
            "info" => Ok(Some(LogLevel::Info)),
            "warning" => Ok(Some(LogLevel::Warn)),
            "error" => Ok(Some(LogLevel::Error)),
            _ => Err(EnvError::InvalidLogLevel { log_level: log_level }),
        }
    }

    fn parse_flaresolverr_url() -> Result<String, EnvError> {
        let Ok(flaresolverr_url) = env::var("FLARESOLVERR_URL") else {
            return Err(EnvError::MissingFlaresolverrUrl);
        };

        if !flaresolverr_url.starts_with("http://") && !flaresolverr_url.starts_with("https://") {
            return Err(EnvError::FlaresolverrUrlNoHTTP {
                url: flaresolverr_url.to_string(),
            });
        }

        Ok(flaresolverr_url.to_string())
    }

    fn parse_webui_port() -> Result<Option<u16>, EnvError> {
        let Ok(port) = env::var("WEBUI_PORT") else {
            return Ok(None);
        };

        let port = port.parse::<u16>().map_err(|_error| return EnvError::InvalidWebUIPort { port: port })?;

        Ok(Some(port))
    }

    fn parse_download_threads() -> Result<Option<u8>, EnvError> {
        let Ok(threads) = env::var("DOWNLOAD_THREADS") else {
            return Ok(None);
        };

        let threads = threads
            .parse::<u8>()
            .map_err(|_error| return EnvError::InvalidThreadCount { thread_count: threads })?;

        Ok(Some(threads))
    }
}

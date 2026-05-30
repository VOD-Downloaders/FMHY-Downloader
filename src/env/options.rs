use std::env;

use thiserror::Error;
use url::Url;

use super::super::logging::LogLevel;

/////////////////////////////////////////////////////
// EnvError
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Error)]
pub enum EnvError {
    #[error("FLARESOLVERR_URL is not set in the current environment.")]
    MissingFlaresolverrUrl,
    #[error("FLARESOLVERR_URL is set to an invalid url \"{url}\", error: {error}")]
    InvalidFlaresolverrUrl { url: String, error: url::ParseError },
    #[error("LOG_LEVEL contains invalid data (must be \"debug\", \"info\", \"warning\" or \"error\". Received: {log_level}")]
    InvalidLogLevel { log_level: String },
    #[error("Expected WEBUI_PORT to be a 16 bit unsigned integer, got: {port}")]
    InvalidWebUIPort { port: String },
    #[error("Expected DOWNLOAD_THREADS to be an 8 bit unsigned integer higher than 0, got: {thread_count}")]
    InvalidThreadCount { thread_count: String },
    #[error("DOWNLOAD_THREADS's thread count {thread_count} exceeds the maximum allowed ({max}).")]
    ThreadCountExceedsMax { thread_count: u8, max: u8 },
    #[error("Expected INDEX_FIND_TIMEOUT to be an 8 bit unsigned integer higher than 0, got: {timeout}")]
    InvalidIndexTimeout { timeout: String },
    #[error("Expected MAX_INDEX_ATTEMPTS to be an 8 bit unsigned integer higher than 0, got: {attempts}")]
    InvalidIndexFindAttempts { attempts: String },
    #[error("Expected SEGMENT_DOWNLOAD_TIMEOUT to be an 8 bit unsigned integer higher than 0, got: {timeout}")]
    InvalidSegmentTimeout { timeout: String },
    #[error("Expected SEGMENT_RETRY_ATTEMPTS to be an 8 bit unsigned integer higher than 0, got: {attempts}")]
    InvalidSegmentRetryAttempts { attempts: String },
}

/////////////////////////////////////////////////////
// Options
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct EnvOptions {
    pub log_level: LogLevel,
    pub flaresolverr_url: Url,
    pub webui_port: u16,

    pub download_threads: u8,

    pub index_find_timeout: u8,
    pub max_index_find_attempts: u8,
    pub segment_download_timeout: u8,
    pub segment_retry_attempts: u8,
}

impl Default for EnvOptions {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            flaresolverr_url: Url::parse("http://flaresolverr:8191/v1").unwrap(),
            webui_port: 8080,

            download_threads: 1,

            index_find_timeout: 7,
            max_index_find_attempts: 5,
            segment_download_timeout: 5,
            segment_retry_attempts: 3,
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

        let index_find_timeout = Self::parse_index_find_timeout()?;
        let max_index_find_attempts = Self::parse_max_index_attempts()?;
        let segment_download_timeout = Self::parse_segment_download_timeout()?;
        let segment_retry_attempts = Self::parse_segment_retry_attempts()?;

        Ok(Self {
            log_level: log_level.unwrap_or(default.log_level),
            flaresolverr_url: flaresolverr_url,
            webui_port: webui_port.unwrap_or(default.webui_port),

            download_threads: download_threads.unwrap_or(default.download_threads),

            index_find_timeout: index_find_timeout.unwrap_or(default.index_find_timeout),
            max_index_find_attempts: max_index_find_attempts.unwrap_or(default.max_index_find_attempts),
            segment_download_timeout: segment_download_timeout.unwrap_or(default.segment_download_timeout),
            segment_retry_attempts: segment_retry_attempts.unwrap_or(default.segment_retry_attempts),
            ..default
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

    fn parse_flaresolverr_url() -> Result<Url, EnvError> {
        let Ok(flaresolverr_url) = env::var("FLARESOLVERR_URL") else {
            return Err(EnvError::MissingFlaresolverrUrl);
        };

        let flaresolverr_url = Url::parse(flaresolverr_url.as_str()).map_err(|error| {
            return EnvError::InvalidFlaresolverrUrl {
                url: flaresolverr_url,
                error: error,
            };
        })?;

        Ok(flaresolverr_url)
    }

    fn parse_webui_port() -> Result<Option<u16>, EnvError> {
        let Ok(port) = env::var("WEBUI_PORT") else {
            return Ok(None);
        };

        let port = port.parse::<u16>().map_err(|_error| EnvError::InvalidWebUIPort { port: port })?;

        Ok(Some(port))
    }

    fn parse_download_threads() -> Result<Option<u8>, EnvError> {
        let Ok(threads) = env::var("DOWNLOAD_THREADS") else {
            return Ok(None);
        };

        let threads = threads
            .parse::<u8>()
            .map_err(|_error| EnvError::InvalidThreadCount { thread_count: threads })?;

        if threads == 0 {
            return Err(EnvError::InvalidThreadCount {
                thread_count: threads.to_string(),
            });
        }

        match std::thread::available_parallelism() {
            Ok(max_threads) => {
                let max_thread_count = max_threads.get() as u8;

                if threads > max_thread_count {
                    return Err(EnvError::ThreadCountExceedsMax {
                        thread_count: threads,
                        max: max_thread_count,
                    });
                }

                Ok(Some(threads))
            },
            Err(error) => {
                warning!("Failed to retrieve maximum thread count with error: {}", error);
                Ok(Some(threads))
            },
        }
    }

    fn parse_index_find_timeout() -> Result<Option<u8>, EnvError> {
        let Ok(index_find_timeout) = env::var("INDEX_FIND_TIMEOUT") else {
            return Ok(None);
        };

        let index_find_timeout = index_find_timeout
            .parse::<u8>()
            .map_err(|_error| EnvError::InvalidIndexTimeout { timeout: index_find_timeout })?;

        if index_find_timeout == 0 {
            return Err(EnvError::InvalidIndexTimeout {
                timeout: index_find_timeout.to_string(),
            });
        }

        Ok(Some(index_find_timeout))
    }

    fn parse_max_index_attempts() -> Result<Option<u8>, EnvError> {
        let Ok(max_index_attempts) = env::var("MAX_INDEX_ATTEMPTS") else {
            return Ok(None);
        };

        let max_index_attempts = max_index_attempts.parse::<u8>().map_err(|_error| EnvError::InvalidIndexFindAttempts {
            attempts: max_index_attempts,
        })?;

        if max_index_attempts == 0 {
            return Err(EnvError::InvalidIndexFindAttempts {
                attempts: max_index_attempts.to_string(),
            });
        }

        Ok(Some(max_index_attempts))
    }

    fn parse_segment_download_timeout() -> Result<Option<u8>, EnvError> {
        let Ok(segment_download_timeout) = env::var("SEGMENT_DOWNLOAD_TIMEOUT") else {
            return Ok(None);
        };

        let segment_download_timeout = segment_download_timeout.parse::<u8>().map_err(|_error| EnvError::InvalidSegmentTimeout {
            timeout: segment_download_timeout,
        })?;

        if segment_download_timeout == 0 {
            return Err(EnvError::InvalidSegmentTimeout {
                timeout: segment_download_timeout.to_string(),
            });
        }

        Ok(Some(segment_download_timeout))
    }

    fn parse_segment_retry_attempts() -> Result<Option<u8>, EnvError> {
        let Ok(segment_retry_attempts) = env::var("SEGMENT_RETRY_ATTEMPTS") else {
            return Ok(None);
        };

        let segment_retry_attempts = segment_retry_attempts
            .parse::<u8>()
            .map_err(|_error| EnvError::InvalidSegmentRetryAttempts {
                attempts: segment_retry_attempts,
            })?;

        if segment_retry_attempts == 0 {
            return Err(EnvError::InvalidSegmentRetryAttempts {
                attempts: segment_retry_attempts.to_string(),
            });
        }

        Ok(Some(segment_retry_attempts))
    }
}

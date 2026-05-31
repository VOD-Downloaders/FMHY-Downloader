use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use thiserror::Error;
use url::Url;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::network::{EnableParams, EventRequestWillBeSent, SetExtraHttpHeadersParams, Headers},
    // cdp::browser_protocol::page::{EventLoadEventFired, NavigateParams},
    error::CdpError,
};
use futures::StreamExt;

use super::super::CHROMIUM_PATH;
use super::super::DownloadStatus;
use super::super::MasterInterceptArguments;
use super::super::super::request;

/////////////////////////////////////////////////////
// IndexError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum MasterInterceptError {
    #[error("Failed to retrieve the domain from \"{url}\"")]
    FailedToRetrieveDomainFromUrl { url: Url },
    #[error("Failed to start browser with error: {error}")]
    FailedToStartBrowser { error: String },
    #[error("Failed to open \"{page}\" with error: {error}")]
    FailedToOpenPage { page: String, error: CdpError },
    #[error("Failed to start monitoring network requests with error: {error}")]
    FailedToStartNetworkMonitoring { error: CdpError },
    #[error("Failed to add custom headers to browser request, error: {error}")]
    FailedToAddCustomHeaders { error: String },
    #[error("Failed to subscribe to network events with error: {error}")]
    FailedToSubsribeToNetworkEvents { error: CdpError },
    #[error("Failed to find the master m3u(8) file before the timeout")]
    FailedToFindMasterM3U,
    #[error("Failed to download master m3u(8) file with error: {error}")]
    FailedToDownloadMasterM3U { error: request::RequestFileError },
    #[error("Failed to read bytes from master m3u(8) file due to error: {error}")]
    FailedToReadMasterM3U { error: String },
    #[error("Failed to read bytes from master m3u(8) file due to error: {error}")]
    FailedToParseMasterM3U { error: String },
    #[error("Failed to download playlist m3u(8) file with error: {error}")]
    FailedToDownloadPlaylistM3U { error: request::RequestFileError },
    #[error("Failed to read bytes from playlist m3u(8) file due to error: {error}")]
    FailedToReadPlaylistM3U { error: String },
    #[error("Found invalid URL \"{url}\" in playlist, error: {error}")]
    FoundInvalidURLInPlaylist { url: String, error: url::ParseError },
}

/////////////////////////////////////////////////////
// MasterData
/////////////////////////////////////////////////////
pub struct MasterData {
    pub resolutions: HashMap<(u32, u32), PlaylistData>,
}

impl MasterData {
    pub async fn get_from(
        url: &Url, arguments: &MasterInterceptArguments, credentials: &request::Credentials, status: Arc<RwLock<DownloadStatus>>,
    ) -> Result<MasterData, MasterInterceptError> {
        trace!("Attempting to get master.m3u(8) file...");
        *status.write().unwrap() = DownloadStatus::FindingMaster { attempt: 1 };

        let referer = {
            let scheme = url.scheme();
            let host = url
                .host_str()
                .ok_or(MasterInterceptError::FailedToRetrieveDomainFromUrl { url: url.clone() })?;

            Url::parse(&format!("{}://{}", scheme, host))
                .map_err(|_error| MasterInterceptError::FailedToRetrieveDomainFromUrl { url: url.clone() })?
        };

        // TODO: Cleanup
        let mut request = None;
        let mut last_error = None;

        for attempt in 1..=arguments.master_attempts {
            *status.write().unwrap() = DownloadStatus::FindingMaster { attempt: attempt };

            match Self::get_master_request(url, arguments, credentials, &referer).await {
                Ok(value) => {
                    request = Some(value);
                    break;
                },
                Err(error) => {
                    warning!("[Attempt {}/{}] Failed to find master.m3u(8) with error: {}", attempt, arguments.master_attempts, error);
                    last_error = Some(error);
                },
            }
        }

        let Some(request) = request else {
            return Err(last_error.unwrap());
        };

        let request_url = Url::parse(request.url.as_str()).unwrap();
        let base_url = request_url.join(".").unwrap(); // Goes up 1 level

        trace!("Sending master retrieval request to \"{}\", base_url: {}, referer: {}.", request_url, base_url, referer);
        *status.write().unwrap() = DownloadStatus::DownloadingMaster;

        let master_data = request::get_file_contents(&request_url, credentials, &referer)
            .map_err(|error| MasterInterceptError::FailedToDownloadMasterM3U { error: error })?;
        let master_contents =
            String::from_utf8(master_data).map_err(|error| MasterInterceptError::FailedToReadMasterM3U { error: error.to_string() })?;

        trace!("Master M3U: {}", master_contents);
        *status.write().unwrap() = DownloadStatus::ParsingMaster;

        let resolutions = Self::parse_master(master_contents.as_str())?;

        let resolutions = {
            let mut temp = HashMap::new();
            for ((width, height), playlist_url) in resolutions {
                temp.insert((width, height), PlaylistData::from(&playlist_url, credentials, &referer, Arc::clone(&status))?);
            }
            temp
        };

        Ok(MasterData { resolutions: resolutions })
    }

    async fn get_master_request(
        url: &Url, arguments: &MasterInterceptArguments, credentials: &request::Credentials, referer: &Url,
    ) -> Result<chromiumoxide::cdp::browser_protocol::network::Request, MasterInterceptError> {
        let user_agent = "--user-agent=".to_string() + credentials.user_agent.as_str();

        let (mut browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .chrome_executable(CHROMIUM_PATH)
                .no_sandbox()
                .new_headless_mode()
                .args(vec![
                    "--disable-setuid-sandbox",
                    "--disable-gpu",
                    "--disable-dev-shm-usage",
                    "--autoplay-policy=no-user-gesture-required",
                    user_agent.as_str(),
                ])
                .build()
                .map_err(|error| MasterInterceptError::FailedToStartBrowser { error: error })?,
        )
        .await
        .map_err(|error| MasterInterceptError::FailedToStartBrowser { error: error.to_string() })?;

        // The handler drives the browser's event loop
        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(error) = event {
                    error!("Failed to handle event with error: {}", error);
                    break;
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|error| MasterInterceptError::FailedToOpenPage {
                page: url.to_string(),
                error: error,
            })?;

        // Start monitoring
        page.execute(EnableParams::default())
            .await
            .map_err(|error| MasterInterceptError::FailedToStartNetworkMonitoring { error: error })?;

        // Subscribe to request events
        let mut requests = page
            .event_listener::<EventRequestWillBeSent>()
            .await
            .map_err(|error| MasterInterceptError::FailedToSubsribeToNetworkEvents { error: error })?;

        // TODO: Maybe add cookies from credentials
        let mut header_map = HashMap::new();
        // header_map.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8");
        // header_map.insert("Accept-Language", "en-GB,en;q=0.9");
        // header_map.insert("Accept-Encoding", "gzip, deflate, br, zstd");
        header_map.insert("Connection", "keep-alive");
        // header_map.insert("Priority", "u=0, i");
        // header_map.insert("Sec-Fetch-Dest", "document");
        // header_map.insert("Sec-Fetch-Mode", "navigate");
        // header_map.insert("Sec-Fetch-Site", "same-origin");
        // header_map.insert("Sec-GPC", "1");
        // header_map.insert("Upgrade-Insecure-Requests", "1");
        header_map.insert("Referer", referer.as_str());

        let headers = Headers::new(serde_json::to_value(header_map).unwrap());

        page.execute(SetExtraHttpHeadersParams::new(headers))
            .await
            .map_err(|error| MasterInterceptError::FailedToAddCustomHeaders { error: error.to_string() })?;

        page.goto(url.as_str()).await.map_err(|error| MasterInterceptError::FailedToOpenPage {
            page: url.to_string(),
            error: error,
        })?;

        // Set a deadline
        let deadline = tokio::time::sleep(std::time::Duration::from_secs(arguments.master_wait_time as u64));
        tokio::pin!(deadline);

        let mut master_request = None;
        loop {
            tokio::select! {
                Some(event) = requests.next() => {
                    let request = &event.request;

                    trace!("{} request to {} captured.", request.method, request.url);

                    if request.method == "GET" && (request.url.ends_with(".m3u") || request.url.ends_with(".m3u8")) {
                        info!("Found master: {}", request.url);
                        master_request = Some(request.clone());
                        break;
                    }
                }
                _ = &mut deadline => {
                    warning!("Capture deadline of {} seconds elapsed before parsing all requests.", arguments.master_wait_time);
                    break;
                }
            }
        }

        let _ = browser.close().await;
        handler_task.abort();

        match master_request {
            Some(request) => Ok(request),
            None => Err(MasterInterceptError::FailedToFindMasterM3U),
        }
    }

    fn parse_master(contents: &str) -> Result<HashMap<(u32, u32), Url>, MasterInterceptError> {
        const DEFAULT_RESOLUTION: (u32, u32) = (1080, 720);

        let mut playlist_map = HashMap::new();
        let mut current_resolution: Option<(u32, u32)> = None;

        for line in contents.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Specification
            if line.starts_with("#EXT-X-STREAM-INF:") {
                // Find the RESOLUTION= attribute within the tag line
                if let Some(resolution_index) = line.find("RESOLUTION=") {
                    let start_value_index = resolution_index + "RESOLUTION=".len();
                    let resolution_str = line[start_value_index..].split(',').next().unwrap_or("");

                    // Split the width and height
                    let mut dimensions = resolution_str.split('x');
                    if let (Some(width_str), Some(height_str)) = (dimensions.next(), dimensions.next()) {
                        let width_str_parse = width_str.parse::<u32>();
                        let height_str_parse = height_str.parse::<u32>();

                        if let (Ok(width), Ok(height)) = (width_str_parse, height_str_parse) {
                            current_resolution = Some((width, height));
                        } else {
                            warning!(
                                "Unable to parse resolution in stream specification. Defaulting to {}x{}",
                                DEFAULT_RESOLUTION.0,
                                DEFAULT_RESOLUTION.1
                            );
                            current_resolution = Some(DEFAULT_RESOLUTION);
                        }
                    }
                } else {
                    warning!("Unable to find resolution in stream specification. Defaulting to {}x{}", DEFAULT_RESOLUTION.0, DEFAULT_RESOLUTION.1);
                    current_resolution = Some(DEFAULT_RESOLUTION);
                }
            }
            // Stream
            else if !line.starts_with('#') {
                if current_resolution.is_none() {
                    warning!("Found stream without resoltion, defaulting to {}x{}", DEFAULT_RESOLUTION.0, DEFAULT_RESOLUTION.1);
                }

                match Url::parse(line) {
                    Ok(url) => {
                        playlist_map.insert(current_resolution.unwrap_or(DEFAULT_RESOLUTION), url);
                    },
                    Err(error) => {
                        warning!("Found invalid stream url in master.m3u(8), error: {}", error);
                    },
                }

                current_resolution = None;
            }
        }

        if playlist_map.is_empty() {
            return Err(MasterInterceptError::FailedToParseMasterM3U {
                error: "No valid URLs found in master.m3u(8) file...".to_string(),
            });
        }

        Ok(playlist_map)
    }
}

/////////////////////////////////////////////////////
// PlaylistData
/////////////////////////////////////////////////////
pub struct PlaylistData {
    pub files: Vec<Url>,
    pub referer: Url,
}

impl PlaylistData {
    pub fn from(
        playlist_url: &Url, credentials: &request::Credentials, referer: &Url, status: Arc<RwLock<DownloadStatus>>,
    ) -> Result<Self, MasterInterceptError> {
        *status.write().unwrap() = DownloadStatus::DownloadingPlaylist;
        let playlist_data = request::get_file_contents(playlist_url, credentials, referer)
            .map_err(|error| MasterInterceptError::FailedToDownloadPlaylistM3U { error: error })?;

        *status.write().unwrap() = DownloadStatus::ParsingPlaylist;
        let playlist_contents =
            String::from_utf8(playlist_data).map_err(|error| MasterInterceptError::FailedToReadPlaylistM3U { error: error.to_string() })?;

        Ok(Self {
            files: Self::parse_playlist(playlist_contents.as_str())?,
            referer: referer.clone(),
        })
    }

    fn parse_playlist(contents: &str) -> Result<Vec<Url>, MasterInterceptError> {
        let segments: Vec<String> = contents
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        let segments_urls = {
            let mut temp = Vec::new();
            for segment in segments {
                let url =
                    Url::parse(segment.as_str()).map_err(|error| MasterInterceptError::FoundInvalidURLInPlaylist { url: segment, error: error })?;
                temp.push(url);
            }
            temp
        };

        Ok(segments_urls)
    }
}

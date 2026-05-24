use core::fmt;
use std::collections::HashMap;

use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::network::{EnableParams, EventRequestWillBeSent, SetExtraHttpHeadersParams, Headers},
    cdp::browser_protocol::page::{EventLoadEventFired, NavigateParams},
    error::CdpError,
};
use futures::StreamExt;
use reqwest::header;

use super::super::request;
use super::super::env;

const CHROMIUM_PATH: &'static str = "/usr/lib/chromium/chromium";

/////////////////////////////////////////////////////
// IndexError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum IndexError {
    FailedToStartBrowser { error: String },
    FailedToOpenPage { page: String, error: CdpError },
    FailedToStartNetworkMonitoring { error: CdpError },
    FailedToAddCustomHeaders { error: String },
    FailedToSubsribeToNetworkEvents { error: CdpError },
    FailedToFindIndexM3U,
    FailedToDownloadIndexM3U { error: request::RequestFileError },
    FailedToReadIndexM3U { error: String },
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IndexError::FailedToStartBrowser { error } => {
                write!(f, "Failed to start browser with error: {}", error)
            },
            IndexError::FailedToOpenPage { page, error } => {
                write!(f, "Failed to open \"{}\" with error: {}", page, error)
            },
            IndexError::FailedToStartNetworkMonitoring { error } => {
                write!(f, "Failed to start monitoring network requests with error: {}", error)
            },
            IndexError::FailedToAddCustomHeaders { error } => {
                write!(f, "Failed to add custom headers to browser request, error: {}", error)
            },
            IndexError::FailedToSubsribeToNetworkEvents { error } => {
                write!(f, "Failed to subscribe to network events with error: {}", error)
            },
            IndexError::FailedToFindIndexM3U => {
                write!(f, "Failed to find the index m3u or m3u8 file before the timeout")
            },
            IndexError::FailedToDownloadIndexM3U { error } => {
                write!(f, "Failed to download index m3u(8) file with error: {}", error)
            },
            IndexError::FailedToReadIndexM3U { error } => {
                write!(f, "Failed to read bytes from m3u file due to error: {}", error)
            },
        }
    }
}

/////////////////////////////////////////////////////
// IndexData
/////////////////////////////////////////////////////
pub struct IndexData {
    pub base_url: String, // Can be empty for index files with full urls
    pub files: Vec<String>,
    pub referer: String,
}

/////////////////////////////////////////////////////
// Index
/////////////////////////////////////////////////////
pub async fn get_index(environment: &env::EnvOptions, url: &str, credentials: &request::Credentials) -> Result<IndexData, IndexError> {
    let referer = url
        .find("://")
        .and_then(|pos| url[pos + 3..].find('/').map(|p| pos + 3 + p))
        .map(|pos| url[..=pos].to_string())
        .unwrap_or_default();

    trace!("Attempting to get index.m3u(8) file...");

    let mut request = None;
    let mut last_error = None;

    for attempt in 1..=environment.max_index_find_attempts {
        match get_index_request(environment, url, credentials, referer.as_str()).await {
            Ok(value) => {
                request = Some(value);
                break;
            },
            Err(error) => {
                warning!("[Attempt {}/{}] Failed to find m3u(8) with error: {}", attempt, environment.max_index_find_attempts, error);
                last_error = Some(error);
            },
        }
    }

    let Some(request) = request else {
        return Err(last_error.unwrap());
    };

    let mut base_url: String = String::new();
    if (request.url.as_str().contains("http://") || request.url.as_str().contains("https://"))
        && let Some(pos) = request.url.as_str().rfind('/')
    {
        base_url = request.url.as_str()[..=pos].to_string();
    }

    trace!("Sending index request to \"{}\", base_url: {}, referer: {}.", url, base_url, referer);

    let index_data = request::get_file_contents(request.url.as_str(), credentials, referer.as_str())
        .map_err(|error| return IndexError::FailedToDownloadIndexM3U { error: error })?;
    let index_contents = String::from_utf8(index_data).map_err(|error| return IndexError::FailedToReadIndexM3U { error: error.to_string() })?;

    trace!("Index M3U: {}", index_contents);

    let urls = parse_index(index_contents.as_str());

    Ok(IndexData {
        base_url: base_url,
        files: urls,
        referer: referer,
    })
}

async fn get_index_request(
    environment: &env::EnvOptions, url: &str, credentials: &request::Credentials, referer: &str,
) -> Result<chromiumoxide::cdp::browser_protocol::network::Request, IndexError> {
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
            .map_err(|error| return IndexError::FailedToStartBrowser { error: error })?,
    )
    .await
    .map_err(|error| IndexError::FailedToStartBrowser { error: error.to_string() })?;

    // The handler drives the browser's event loop
    let handler_task = tokio::spawn(async move {
        while let Some(e) = handler.next().await {
            if e.is_err() {
                error!("Failed to handle event with error: {}", e.unwrap_err());
                break;
            }
        }
    });

    let page = browser.new_page("about:blank").await.map_err(|error| {
        return IndexError::FailedToOpenPage {
            page: url.to_string(),
            error: error,
        };
    })?;

    // Start monitoring
    page.execute(EnableParams::default()).await.map_err(|error| {
        return IndexError::FailedToStartNetworkMonitoring { error: error };
    })?;

    // Subscribe to request events
    let mut requests = page
        .event_listener::<EventRequestWillBeSent>()
        .await
        .map_err(|error| return IndexError::FailedToSubsribeToNetworkEvents { error: error })?;

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
    header_map.insert("Referer", referer);

    let headers = Headers::new(serde_json::to_value(header_map).unwrap());

    page.execute(SetExtraHttpHeadersParams::new(headers))
        .await
        .map_err(|error| IndexError::FailedToAddCustomHeaders { error: error.to_string() })?;

    page.goto(url).await.map_err(|error| IndexError::FailedToOpenPage {
        page: url.to_string(),
        error: error,
    })?;

    // Set a deadline
    let deadline = tokio::time::sleep(std::time::Duration::from_secs(environment.index_find_timeout as u64));
    tokio::pin!(deadline);

    let mut index_request = None;
    loop {
        tokio::select! {
            Some(event) = requests.next() => {
                let request = &event.request;

                trace!("{} request to {} captured.", request.method, request.url);

                if request.method == "GET" && request.url.contains(".m3u") {
                    info!("Found stream: {}", request.url);
                    index_request = Some(request.clone());
                    break;
                }
            }
            _ = &mut deadline => {
                warning!("Capture deadline of {} seconds elapsed before parsing all requests.", environment.index_find_timeout);
                break;
            }
        }
    }

    let _ = browser.close().await;
    handler_task.abort();

    match index_request {
        Some(request) => Ok(request),
        None => Err(IndexError::FailedToFindIndexM3U),
    }
}

fn parse_index(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

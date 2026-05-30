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

use super::super::super::env;
use super::super::super::request;

const CHROMIUM_PATH: &str = "/usr/lib/chromium/chromium";

/////////////////////////////////////////////////////
// IndexError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum IndexError {
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
    #[error("Failed to find the index m3u or m3u8 file before the timeout")]
    FailedToFindIndexM3U,
    #[error("Failed to download index m3u(8) file with error: {error}")]
    FailedToDownloadIndexM3U { error: request::RequestFileError },
    #[error("Failed to read bytes from m3u file due to error: {error}")]
    FailedToReadIndexM3U { error: String },
}

/////////////////////////////////////////////////////
// IndexData
/////////////////////////////////////////////////////
pub struct IndexData {
    pub base_url: Url,
    pub files: Vec<String>,
    pub referer: Url,
}

/////////////////////////////////////////////////////
// Index
/////////////////////////////////////////////////////
pub async fn get_index(environment: &env::EnvOptions, url: &Url, credentials: &request::Credentials) -> Result<IndexData, IndexError> {
    trace!("Attempting to get index.m3u(8) file...");

    let referer = {
        let scheme = url.scheme();
        let host = url.host_str().ok_or(IndexError::FailedToRetrieveDomainFromUrl { url: url.clone() })?;

        Url::parse(&format!("{}://{}", scheme, host)).map_err(|_| IndexError::FailedToRetrieveDomainFromUrl { url: url.clone() })?
    };

    let mut request = None;
    let mut last_error = None;

    for attempt in 1..=environment.max_index_find_attempts {
        match get_index_request(environment, url, credentials, &referer).await {
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

    let request_url = Url::parse(request.url.as_str()).unwrap();
    let base_url = request_url.join(".").unwrap(); // Goes up 1 level

    trace!("Sending index retrieval request to \"{}\", base_url: {}, referer: {}.", request_url, base_url, referer);

    let index_data =
        request::get_file_contents(&request_url, credentials, &referer).map_err(|error| IndexError::FailedToDownloadIndexM3U { error: error })?;
    let index_contents = String::from_utf8(index_data).map_err(|error| IndexError::FailedToReadIndexM3U { error: error.to_string() })?;

    trace!("Index M3U: {}", index_contents);

    let urls = parse_index(index_contents.as_str());

    Ok(IndexData {
        base_url: base_url,
        files: urls,
        referer: referer,
    })
}

async fn get_index_request(
    environment: &env::EnvOptions, url: &Url, credentials: &request::Credentials, referer: &Url,
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
            .map_err(|error| IndexError::FailedToStartBrowser { error: error })?,
    )
    .await
    .map_err(|error| IndexError::FailedToStartBrowser { error: error.to_string() })?;

    // The handler drives the browser's event loop
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(error) = event {
                error!("Failed to handle event with error: {}", error);
                break;
            }
        }
    });

    let page = browser.new_page("about:blank").await.map_err(|error| IndexError::FailedToOpenPage {
        page: url.to_string(),
        error: error,
    })?;

    // Start monitoring
    page.execute(EnableParams::default())
        .await
        .map_err(|error| IndexError::FailedToStartNetworkMonitoring { error: error })?;

    // Subscribe to request events
    let mut requests = page
        .event_listener::<EventRequestWillBeSent>()
        .await
        .map_err(|error| IndexError::FailedToSubsribeToNetworkEvents { error: error })?;

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
        .map_err(|error| IndexError::FailedToAddCustomHeaders { error: error.to_string() })?;

    page.goto(url.as_str()).await.map_err(|error| IndexError::FailedToOpenPage {
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

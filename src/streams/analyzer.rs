use std::any::Any;
use std::collections::HashMap;

use thiserror::Error;
use url::Url;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::network::{
        RequestId, EnableParams, EventRequestWillBeSent, EventResponseReceived, SetExtraHttpHeadersParams, GetResponseBodyParams, Headers,
    },
    error::CdpError,
};
use futures::StreamExt;
use base64::{engine::general_purpose, Engine};

use super::super::request::Requester;

const CHROMIUM_PATH: &str = "/usr/lib/chromium/chromium";

/////////////////////////////////////////////////////
// BrowserRequest
/////////////////////////////////////////////////////
pub type BrowserRequest = chromiumoxide::cdp::browser_protocol::network::Request;
pub type BrowserResponse = chromiumoxide::cdp::browser_protocol::network::Response;

/////////////////////////////////////////////////////
// AnalyzeError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum AnalyzeError {
    #[error("Failed to start browser with error: {0}")]
    FailedToStartBrowser(String),
    #[error("Failed to open \"{0}\" with error: {1}")]
    FailedToOpenPage(Url, CdpError),
    #[error("Failed to start monitoring network requests with error: {0}")]
    FailedToStartNetworkMonitoring(CdpError),
    #[error("Failed to add custom headers to browser request, error: {0}")]
    FailedToAddCustomHeaders(CdpError),
    #[error("Failed to subscribe to network events with error: {0}")]
    FailedToSubsribeToNetworkEvents(CdpError),
}

/////////////////////////////////////////////////////
// Analyzer
/////////////////////////////////////////////////////
#[async_trait::async_trait]
pub trait Analyzer: Any + Send + Sync {
    // NOTE: When returning true this analyzer signals it's done analyzing requests and may stop early
    async fn analyze(&mut self, requester: &Requester, request: &BrowserRequest, response: Option<&BrowserResponse>, body: Option<String>) -> bool;

    // fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/////////////////////////////////////////////////////
// Analyze URL
/////////////////////////////////////////////////////
pub async fn analyze_url(url: &Url, requester: &Requester, analyzers: &mut [Box<dyn Analyzer>], analyze_duration: u64) -> Result<(), AnalyzeError> {
    let requester_specification = requester.get_specification();

    trace!(
        "Starting analysis on \"{}\" with user_agent: {}, headers: {:?}",
        url, requester_specification.user_agent, requester_specification.headers
    );

    let mut analyzers_copy: Vec<&mut Box<dyn Analyzer>> = analyzers.iter_mut().collect();

    let user_agent = "user-agent=".to_string() + requester_specification.user_agent.as_str();

    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .chrome_executable(CHROMIUM_PATH)
            .no_sandbox()
            .with_head() // Force to run browser with head (in Xvfb)
            .args(vec![
                "disable-setuid-sandbox",
                // "disable-gpu", // The browser runs in Xvfb
                "disable-dev-shm-usage",
                "autoplay-policy=no-user-gesture-required",
                "disable-blink-features=AutomationControlled", // Removes navigator.webdriver flag
                user_agent.as_str(),
            ])
            .build()
            .map_err(AnalyzeError::FailedToStartBrowser)?,
    )
    .await
    .map_err(|error| AnalyzeError::FailedToStartBrowser(error.to_string()))?;

    // The handler drives the browser's event loop
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(error) = event {
                error!("Failed to handle browser event with error: {}", error);
                break;
            }
        }
    });

    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|error| AnalyzeError::FailedToOpenPage(url.clone(), error))?;

    // Start monitoring
    page.execute(EnableParams::default())
        .await
        .map_err(AnalyzeError::FailedToStartNetworkMonitoring)?;

    // Subscribe to events
    let mut requests = page
        .event_listener::<EventRequestWillBeSent>()
        .await
        .map_err(AnalyzeError::FailedToSubsribeToNetworkEvents)?;

    let mut responses = page
        .event_listener::<EventResponseReceived>()
        .await
        .map_err(AnalyzeError::FailedToSubsribeToNetworkEvents)?;

    let headers = Headers::new(serde_json::Value::Object(
        requester_specification
            .headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), serde_json::Value::String(v.to_string()))))
            .collect::<serde_json::Map<_, _>>(),
    ));
    page.execute(SetExtraHttpHeadersParams::new(headers))
        .await
        .map_err(AnalyzeError::FailedToAddCustomHeaders)?;

    // Open actual url
    page.goto(url.as_str())
        .await
        .map_err(|error| AnalyzeError::FailedToOpenPage(url.clone(), error))?;

    // Set a deadline
    let deadline = tokio::time::sleep(std::time::Duration::from_secs(analyze_duration));
    tokio::pin!(deadline);

    let mut pending: HashMap<RequestId, BrowserRequest> = HashMap::new();

    loop {
        tokio::select! {
            Some(event) = requests.next() => {
                let request = &event.request;
                trace!("{} request to {} captured. Headers: {:?}", request.method, request.url, request.headers);
                pending.insert(event.request_id.clone(), request.clone());
            },
            Some(event) = responses.next() => {
                let response = &event.response;
                trace!("Response from {} captured.", response.url);

                if let Some(request) = pending.remove(&event.request_id) {
                    let body = page
                        .execute(GetResponseBodyParams::new(event.request_id.clone()))
                        .await
                        .ok()
                        .map(|body| {
                            // Decode base64 if encoded
                            if body.base64_encoded {
                                general_purpose::STANDARD
                                    .decode(&body.body)
                                    .ok()
                                    .and_then(|bytes| String::from_utf8(bytes).ok())
                                    .unwrap_or(body.body.clone())
                            } else {
                                body.body.clone()
                            }
                        });

                    // trace!("Response body from \"{}\" captured: {:?}", response.url, body);
                    run_analyzers(&mut analyzers_copy, requester, &request, Some(response), body.clone()).await;
                }

                if analyzers_copy.is_empty() {
                    break;
                }
            },
            _ = &mut deadline => {
                break;
            }
        }
    }

    // Handle requests that haven't gotten a response yet
    for (_id, request) in pending {
        run_analyzers(&mut analyzers_copy, requester, &request, None, None).await;
    }

    let _ = browser.close().await;
    handler_task.abort();

    Ok(())
}

async fn run_analyzers(
    analyzers: &mut Vec<&mut Box<dyn Analyzer>>, requester: &Requester, request: &BrowserRequest, response: Option<&BrowserResponse>,
    body: Option<String>,
) {
    let done = futures::future::join_all(analyzers.iter_mut().map(|a| a.analyze(requester, request, response, body.clone()))).await;
    let mut done = done.into_iter();
    analyzers.retain(|_| !done.next().unwrap());
}

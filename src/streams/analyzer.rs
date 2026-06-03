use thiserror::Error;
use url::Url;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::network::{EnableParams, EventRequestWillBeSent, SetExtraHttpHeadersParams, Headers},
    // cdp::browser_protocol::page::{EventLoadEventFired, NavigateParams},
    error::CdpError,
};
use futures::StreamExt;

use super::super::request::RequesterSpecification;

const CHROMIUM_PATH: &str = "/usr/lib/chromium/chromium";

/////////////////////////////////////////////////////
// BrowserRequest
/////////////////////////////////////////////////////
pub type BrowserRequest = chromiumoxide::cdp::browser_protocol::network::Request;

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
pub trait Analyzer {
    fn analyze(&mut self, request: &BrowserRequest);
}

/////////////////////////////////////////////////////
// Analyze URL
/////////////////////////////////////////////////////
pub async fn analyze_url(
    url: &Url, specification: RequesterSpecification, mut analyzers: Vec<Box<dyn Analyzer>>, analyze_duration: u64,
) -> Result<(), AnalyzeError> {
    let user_agent = "--user-agent=".to_string() + specification.user_agent.as_str();

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

    // Subscribe to request events
    let mut requests = page
        .event_listener::<EventRequestWillBeSent>()
        .await
        .map_err(AnalyzeError::FailedToStartNetworkMonitoring)?;

    let headers = Headers::new(serde_json::Value::Object(
        specification
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

    loop {
        tokio::select! {
            Some(event) = requests.next() => {
                let request = &event.request;
                trace!("{} request to {} captured.", request.method, request.url);

                for analyzer in &mut analyzers {
                    analyzer.analyze(request);
                }
            }
            _ = &mut deadline => {
                break;
            }
        }
    }

    let _ = browser.close().await;
    handler_task.abort();

    Ok(())
}

use thiserror::Error;
use url::Url;

use super::super::Analyzer;
use super::super::BrowserRequest;
use super::super::BrowserResponse;

/////////////////////////////////////////////////////
// M3URequest
/////////////////////////////////////////////////////
pub struct M3URequest {
    pub url: Url,
    pub contents: String,
}

/////////////////////////////////////////////////////
// M3UAnalyzer
/////////////////////////////////////////////////////
pub struct M3UAnalyzer {
    pub requests: Vec<M3URequest>,
}

impl M3UAnalyzer {
    pub fn new() -> Self {
        M3UAnalyzer { requests: Vec::new() }
    }
}

impl Analyzer for M3UAnalyzer {
    fn analyze(&mut self, request: &BrowserRequest, response: &BrowserResponse, body: Option<String>) -> bool {
        const M3U8_MIMES: &[&str] = &[
            "application/vnd.apple.mpegurl", // official HLS/M3U8
            "application/x-mpegurl",         // common non-standard variant
            "audio/mpegurl",                 // shared with M3U
            "audio/x-mpegurl",               // legacy M3U, but used for M3U8 too
        ];

        if !(request.method == "GET"
            && ((request.url.ends_with(".m3u") || request.url.ends_with(".m3u8")) || (*M3U8_MIMES).contains(&response.mime_type.as_str())))
        {
            return false;
        }

        let body_str = body.unwrap_or_default();
        trace!("Found M3U request: GET {} (mime-type: {}), with response: {}", request.url, response.mime_type.as_str(), body_str.as_str());

        self.requests.push(M3URequest {
            url: Url::parse(request.url.as_str()).unwrap(),
            contents: body_str,
        });

        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

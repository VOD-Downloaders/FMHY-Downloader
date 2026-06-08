use url::Url;

use super::super::super::request;
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

#[async_trait::async_trait]
impl Analyzer for M3UAnalyzer {
    async fn analyze(
        &mut self, requester: &request::Requester, request: &BrowserRequest, response: Option<&BrowserResponse>, body: Option<String>,
    ) -> bool {
        const M3U8_MIMES: &[&str] = &[
            "application/vnd.apple.mpegurl", // official HLS/M3U8
            "application/x-mpegurl",         // common non-standard variant
            "audio/mpegurl",                 // shared with M3U
            "audio/x-mpegurl",               // legacy M3U, but used for M3U8 too
        ];

        if !(request.method == "GET" && (request.url.ends_with(".m3u") || request.url.ends_with(".m3u8"))) {
            if let Some(response) = response {
                if !((*M3U8_MIMES).contains(&response.mime_type.as_str())) {
                    return false;
                }
                // Escapes if the response is M3U8_MIME
            } else {
                return false;
            }
        }

        let body_str = {
            let mut body_str = body.unwrap_or_default();
            if body_str.is_empty() {
                trace!("Body from M3U response was empty, manually retrying...");

                let url = Url::parse(request.url.as_str()).unwrap();
                let result = requester.get_file_contents(&url, None).await;
                let Ok(bytes) = result else {
                    error!("Failed to get body from \"{}\" due to error: {}", url, result.unwrap_err());
                    return false;
                };

                let result = String::from_utf8(bytes);
                if let Err(error) = result {
                    error!("Failed to convert response bytes to body string, error: {}", error);
                    return false;
                }

                body_str = result.unwrap();
            }
            body_str
        };

        trace!("Found M3U request: GET {}, with response: {}", request.url, body_str.as_str());

        self.requests.push(M3URequest {
            url: Url::parse(request.url.as_str()).unwrap(),
            contents: body_str,
        });

        false
    }

    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

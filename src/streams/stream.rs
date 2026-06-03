use chromiumoxide::cdp::browser_protocol::input;
use url::Url;
use serde::{Serialize, Deserialize};

use super::analyze_url;
use super::Analyzer;
use super::M3UResult;
use super::M3UAnalyzer;
use super::parse_m3u_contents;
use super::super::config;
use super::super::config::DownloadMethod;
use super::super::request::RequesterSpecification;

/////////////////////////////////////////////////////
// StreamType
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum StreamType {
    M3U(Vec<Url>),
}

impl Default for StreamType {
    fn default() -> Self {
        StreamType::M3U(Vec::new())
    }
}

/////////////////////////////////////////////////////
// Stream
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct Stream {
    pub width: u32,
    pub height: u32,

    #[serde(skip)]
    pub stream_type: StreamType,
}

/////////////////////////////////////////////////////
// Streams
/////////////////////////////////////////////////////
pub async fn get_streams(indexer: &config::Indexer, request_specification: &RequesterSpecification, input_url: &Url) -> Vec<Stream> {
    match &indexer.download.method {
        DownloadMethod::IndexInterception(specification) => {
            for attempt in 1..=specification.retries {
                let mut analyzers: Vec<Box<dyn Analyzer>> = vec![Box::new(M3UAnalyzer::new())];

                let result = analyze_url(input_url, request_specification, &mut analyzers, specification.wait_time as u64).await;
                if let Err(error) = result {
                    error!("[Attempt {}/{}] Analyzing requests for {} failed with error: {}", attempt, specification.retries, input_url, error);
                    continue;
                }

                let m3u_analyzer = analyzers[0].as_any_mut().downcast_mut::<M3UAnalyzer>().unwrap();

                for request in m3u_analyzer.requests.drain(..) {
                    let parse_result = parse_m3u_contents(request.contents.as_str());
                    let Ok(result) = parse_result else {
                        error!("[Attempt {}/{}] Parsing M3U faild with error: {}", attempt, specification.retries, parse_result.unwrap_err());
                        continue;
                    };

                    if let M3UResult::Index(segments) = result {
                        let mut url_segments = Vec::with_capacity(segments.len());

                        for segment in segments {
                            let segment_url = Url::parse(segment.as_str());
                            let full_url = {
                                match segment_url {
                                    Ok(url) => url,
                                    Err(_error) => indexer.url.join(segment.as_str()).unwrap(),
                                }
                            };
                            url_segments.push(full_url);
                        }

                        return vec![Stream {
                            width: M3UResult::DEFAULT_RESOLUTION.0,
                            height: M3UResult::DEFAULT_RESOLUTION.1,
                            stream_type: StreamType::M3U(url_segments),
                        }];
                    }
                }
            }

            Vec::new()
        },
        DownloadMethod::MasterInterception(specification) => {
            todo!()
        },
        DownloadMethod::MP4Interception(specification) => {
            todo!()
        },
    }
}

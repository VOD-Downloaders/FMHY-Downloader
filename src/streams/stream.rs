use url::Url;
use serde::{Serialize, Deserialize};

use super::analyze_url;
use super::Analyzer;
use super::M3UResult;
use super::M3UAnalyzer;
use super::parse_m3u_contents;
use super::super::config;
use super::super::config::DownloadMethod;
use super::super::request::Requester;

/////////////////////////////////////////////////////
// StreamType
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
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

    pub stream_type: StreamType,
}

/////////////////////////////////////////////////////
// Streams
/////////////////////////////////////////////////////
pub async fn get_streams(indexer: &config::Indexer, requester: &Requester, input_url: &Url) -> Vec<Stream> {
    match &indexer.download.method {
        DownloadMethod::IndexInterception(specification) => {
            for attempt in 1..=specification.retries {
                let mut analyzers: Vec<Box<dyn Analyzer>> = vec![Box::new(M3UAnalyzer::new())];

                let result = analyze_url(input_url, requester.get_specification(), &mut analyzers, specification.wait_time as u64).await;
                if let Err(error) = result {
                    error!("[Attempt {}/{}] Analyzing requests for {} failed with error: {}", attempt, specification.retries, input_url, error);
                    continue;
                }

                let m3u_analyzer = analyzers[0].as_any_mut().downcast_mut::<M3UAnalyzer>().unwrap();

                for request in m3u_analyzer.requests.drain(..) {
                    trace!("Analyzing request to \"{}\" for index m3u(8).", request.url);

                    let parse_result = parse_m3u_contents(request.contents.as_str());
                    let Ok(result) = parse_result else {
                        error!("[Attempt {}/{}] Parsing M3U faild with error: {}", attempt, specification.retries, parse_result.unwrap_err());
                        continue;
                    };

                    if let M3UResult::Index(segments) = result {
                        return vec![create_stream_from_segments(segments, input_url, M3UResult::DEFAULT_RESOLUTION)];
                    }
                }
            }

            Vec::new()
        },
        DownloadMethod::MasterInterception(specification) => {
            for attempt in 1..=specification.retries {
                let mut analyzers: Vec<Box<dyn Analyzer>> = vec![Box::new(M3UAnalyzer::new())];

                let result = analyze_url(input_url, requester.get_specification(), &mut analyzers, specification.wait_time as u64).await;
                if let Err(error) = result {
                    error!("[Attempt {}/{}] Analyzing requests for {} failed with error: {}", attempt, specification.retries, input_url, error);
                    continue;
                }

                let m3u_analyzer = analyzers[0].as_any_mut().downcast_mut::<M3UAnalyzer>().unwrap();

                for request in m3u_analyzer.requests.drain(..) {
                    trace!("Analyzing request to \"{}\" for master m3u(8).", request.url);

                    let parse_result = parse_m3u_contents(request.contents.as_str());
                    let Ok(result) = parse_result else {
                        error!("[Attempt {}/{}] Parsing M3U faild with error: {}", attempt, specification.retries, parse_result.unwrap_err());
                        continue;
                    };

                    if let M3UResult::Master(indexes) = result {
                        let mut streams = Vec::new();

                        for ((width, height), index_url) in indexes {
                            trace!("Sending request to \"{}\" for index m3u(8).", index_url);

                            let index_m3u_bytes = requester.get_file_contents(&index_url, None).await;
                            let Ok(index_m3u_bytes) = index_m3u_bytes else {
                                error!("Failed to get index m3u response from \"{}\" due to error: {}", index_url, index_m3u_bytes.unwrap_err());
                                continue;
                            };

                            let index_m3u_str = String::from_utf8(index_m3u_bytes);
                            let Ok(index_m3u_str) = index_m3u_str else {
                                error!("Failed to read index m3u with error: {}", index_m3u_str.unwrap_err());
                                continue;
                            };

                            trace!("Analyzing request to \"{}\" for index m3u(8).", index_url);

                            let index_m3u = parse_m3u_contents(index_m3u_str.as_str());
                            let Ok(index_m3u) = index_m3u else {
                                error!("Failed to parse index m3u with error: {}", index_m3u.unwrap_err());
                                continue;
                            };

                            if let M3UResult::Index(segments) = index_m3u {
                                streams.push(create_stream_from_segments(segments, input_url, (width, height)));
                            } else {
                                warning!("Expected m3u contents from \"{}\" to be an index file...", index_url);
                            }
                        }

                        if !streams.is_empty() {
                            return streams;
                        }
                    }
                }
            }

            Vec::new()
        },
        DownloadMethod::MP4Interception(specification) => {
            todo!()
        },
    }
}

fn create_stream_from_segments(segments: Vec<String>, input_url: &Url, resolution: (u32, u32)) -> Stream {
    let mut url_segments = Vec::with_capacity(segments.len());

    for segment in segments {
        let segment_url = Url::parse(segment.as_str());
        let full_url = {
            match segment_url {
                Ok(url) => url,
                Err(_error) => input_url.join(".").unwrap().join(segment.as_str()).unwrap(),
            }
        };
        url_segments.push(full_url);
    }

    Stream {
        width: resolution.0,
        height: resolution.1,
        stream_type: StreamType::M3U(url_segments),
    }
}

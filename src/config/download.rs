use url::Url;
use serde::{Serialize, Deserialize};

use super::super::request::HeaderMap;

/////////////////////////////////////////////////////
// DownloadSpecifications
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDownloadSpecification {
    pub wait_time: u8,
    pub retries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDownloadSpecification {
    pub wait_time: u8,
    pub retries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MP4DownloadSpecification {
    pub wait_time: u32,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadMethod {
    #[serde(rename = "index")]
    IndexInterception(IndexDownloadSpecification),

    #[serde(rename = "master")]
    MasterInterception(MasterDownloadSpecification),

    #[serde(rename = "mp4")]
    MP4Interception(MP4DownloadSpecification),
}

/////////////////////////////////////////////////////
// DownloadSpecifications
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreDownloadSpecifiation {
    pub segment_timeout: u32,
    pub segment_attempts: u32,
    pub headers: HeaderMap,
}

impl PreDownloadSpecifiation {
    pub fn resolve_variables(&mut self, base_url: &Url, stream_url: &Url) {
        for (_header_name, header_value) in &mut self.headers {
            if let Ok(header_str) = header_value.to_str() {
                let mut header_str = header_str.to_string();

                header_str = header_str.replace("{base_url}", base_url.as_str());
                header_str = header_str.replace("{stream_url}", stream_url.as_str());

                if let Ok(value) = reqwest::header::HeaderValue::from_str(header_str.as_str()) {
                    *header_value = value;
                }
            }
        }
    }
}

impl Default for PreDownloadSpecifiation {
    fn default() -> Self {
        Self {
            segment_timeout: 5,
            segment_attempts: 5,
            headers: HeaderMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDownloadSpecification {
    pub remove_front_bytes: u32,
    pub remove_back_bytes: u32,
}

impl Default for PostDownloadSpecification {
    fn default() -> Self {
        Self {
            remove_front_bytes: 0,
            remove_back_bytes: 0,
        }
    }
}

/////////////////////////////////////////////////////
// DownloadSpecification
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSpecification {
    pub method: DownloadMethod,
    pub segment_pre_download: PreDownloadSpecifiation,
    pub segment_post_download: PostDownloadSpecification,
}

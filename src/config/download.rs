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
pub struct SegmentDownloadSpecifiation {
    pub segment_timeout: u32,
    pub segment_attempts: u32,
    pub headers: HeaderMap,
}

impl Default for SegmentDownloadSpecifiation {
    fn default() -> Self {
        Self {
            segment_timeout: 5,
            segment_attempts: 5,
            headers: HeaderMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPostDownloadSpecification {
    pub remove_front_bytes: u32,
    pub remove_back_bytes: u32,
}

impl Default for SegmentPostDownloadSpecification {
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
    pub segment_download: SegmentDownloadSpecifiation,
    pub segment_post_download: SegmentPostDownloadSpecification,
}

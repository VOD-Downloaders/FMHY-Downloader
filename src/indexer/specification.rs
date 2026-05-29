use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// Specifications
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexSpecification {
    pub wait_time: u32,
    pub retries: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MP4Specification {
    pub wait_time: u32,
    pub retries: u32,
}

/////////////////////////////////////////////////////
// IndexerType
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IndexerType {
    #[serde(rename = "index")]
    IndexInterception(IndexSpecification),

    #[serde(rename = "mp4")]
    MP4Interception(MP4Specification),
}

/////////////////////////////////////////////////////
// Indexer
/////////////////////////////////////////////////////
pub struct Indexer {}

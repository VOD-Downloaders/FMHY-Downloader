use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// IndexerType
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IndexerType {
    #[serde(rename = "index")]
    IndexInterception {
        #[serde(default = "default_wait_time")]
        wait_time: u8,
        #[serde(default = "default_retries")]
        retries: u8,
    },

    #[serde(rename = "mp4")]
    MP4Interception,
}

fn default_wait_time() -> u8 {
    6
}

fn default_retries() -> u8 {
    5
}

/////////////////////////////////////////////////////
// Indexer
/////////////////////////////////////////////////////
pub struct Indexer {}

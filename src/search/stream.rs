use url::Url;
use serde::{Serialize, Deserialize};

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
    pub quality: String,
    pub stream_type: StreamType,
}

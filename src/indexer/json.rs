use core::fmt;

use url::Url;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};

use super::IndexerType;

/////////////////////////////////////////////////////
// Serializing
/////////////////////////////////////////////////////
impl Serialize for IndexerType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let str = match self {
            IndexerType::IndexInterception => "index",
            IndexerType::MP4Interception => "mp4",
        };
        serializer.serialize_str(str)
    }
}

struct IndexerTypeVisitor;

impl<'de> Visitor<'de> for IndexerTypeVisitor {
    type Value = IndexerType;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a string of either \"index\" or \"mp4\"")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<IndexerType, E> {
        match value {
            "index" => Ok(IndexerType::IndexInterception),
            "mp4" => Ok(IndexerType::MP4Interception),
            other => Err(E::unknown_variant(other, &["index", "mp4"])),
        }
    }
}

impl<'de> Deserialize<'de> for IndexerType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(IndexerTypeVisitor)
    }
}

/////////////////////////////////////////////////////
// IndexerBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerBody {
    pub url: Url,
    pub uses_cloudflare: bool,

    #[serde(rename = "type")]
    pub indexer_type: IndexerType,
}

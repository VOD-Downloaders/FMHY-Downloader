use core::fmt;

use url::Url;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};

use super::IndexerType;

/////////////////////////////////////////////////////
// IndexerBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerBody {
    pub url: Url,
    pub uses_cloudflare: bool,

    pub specification: IndexerType,
}

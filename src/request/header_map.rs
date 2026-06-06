use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use reqwest::header::{HeaderName, HeaderValue, IntoIter, Iter, IterMut};
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Error as SerError, Serialize, SerializeMap, Serializer};

/////////////////////////////////////////////////////
// HeaderMapExt
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Default)]
pub struct HeaderMapExt(pub reqwest::header::HeaderMap);

impl HeaderMapExt {
    pub fn new() -> Self {
        Self(reqwest::header::HeaderMap::new())
    }

    pub fn into_inner(self) -> reqwest::header::HeaderMap {
        self.0
    }
}

impl Deref for HeaderMapExt {
    type Target = reqwest::header::HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HeaderMapExt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<reqwest::header::HeaderMap> for HeaderMapExt {
    fn from(map: reqwest::header::HeaderMap) -> Self {
        Self(map)
    }
}

impl From<HeaderMapExt> for reqwest::header::HeaderMap {
    fn from(ext: HeaderMapExt) -> Self {
        ext.0
    }
}

impl<'headermap> IntoIterator for &'headermap HeaderMapExt {
    type Item = (&'headermap HeaderName, &'headermap HeaderValue);
    type IntoIter = Iter<'headermap, HeaderValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'headermap> IntoIterator for &'headermap mut HeaderMapExt {
    type Item = (&'headermap HeaderName, &'headermap mut HeaderValue);
    type IntoIter = IterMut<'headermap, HeaderValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for HeaderMapExt {
    type Item = (Option<HeaderName>, HeaderValue);
    type IntoIter = IntoIter<HeaderValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/////////////////////////////////////////////////////
// Serialize & Deserialize
/////////////////////////////////////////////////////
impl Serialize for HeaderMapExt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.keys_len()))?;
        for (name, value) in &self.0 {
            let value = value.to_str().map_err(SerError::custom)?;
            map.serialize_entry(name.as_str(), value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for HeaderMapExt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = BTreeMap::<String, String>::deserialize(deserializer)?;
        let mut headers = reqwest::header::HeaderMap::with_capacity(entries.len());

        for (name, value) in entries {
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(DeError::custom)?;
            let value = HeaderValue::from_str(value.as_str()).map_err(DeError::custom)?;
            headers.insert(name, value);
        }

        Ok(Self(headers))
    }
}

/////////////////////////////////////////////////////
// Type
/////////////////////////////////////////////////////
pub type HeaderMap = HeaderMapExt;

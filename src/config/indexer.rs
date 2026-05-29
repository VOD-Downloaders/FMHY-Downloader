use std::path::Path;
use std::path::PathBuf;

use url::Url;
use thiserror::Error;
use futures::TryFutureExt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// IndexerBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerBody {
    pub url: Url,
    pub uses_cloudflare: bool,

    pub specification: Indexer,
}

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
pub enum Indexer {
    #[serde(rename = "index")]
    IndexInterception(IndexSpecification),

    #[serde(rename = "mp4")]
    MP4Interception(MP4Specification),
}

/////////////////////////////////////////////////////
// ParseIndexError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum ParseIndexError {
    #[error("File \"{0}\" doesn't exist.")]
    FileDoesntExist(PathBuf),
    #[error("Unable to read state.json with error: {0}")]
    UnableToReadFile(std::io::Error),
    #[error("Unable to parse json \"{json}\" due to error: {error}")]
    UnableToParseJson { json: String, error: serde_json::Error },
}

pub async fn parse_indexer_from_file(file: &Path) -> Result<Indexer, ParseIndexError> {
    if !file.exists() {
        return Err(ParseIndexError::FileDoesntExist(file.to_path_buf()));
    }

    let contents = tokio::fs::read_to_string(file).map_err(ParseIndexError::UnableToReadFile).await?;

    let json_body: Indexer = serde_json::from_str(contents.as_str()).map_err(|error| ParseIndexError::UnableToParseJson {
        json: contents,
        error: error,
    })?;

    Ok(json_body)
}

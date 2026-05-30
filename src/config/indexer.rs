use std::path::Path;
use std::path::PathBuf;

use url::Url;
use thiserror::Error;
use futures::TryFutureExt;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// IndexerBody
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    pub name: String,
    pub url: Url,
    pub mirrors: Vec<Url>,

    pub uses_cloudflare: bool,

    pub download: DownloadMethod,
}

/////////////////////////////////////////////////////
// Specifications
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDownloadSpecification {
    pub wait_time: u32,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MP4DownloadSpecification {
    pub wait_time: u32,
    pub retries: u32,
}

/////////////////////////////////////////////////////
// IndexerType
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadMethod {
    #[serde(rename = "index")]
    IndexInterception(IndexDownloadSpecification),

    #[serde(rename = "mp4")]
    MP4Interception(MP4DownloadSpecification),
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

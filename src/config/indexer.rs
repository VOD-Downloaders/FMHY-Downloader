use std::path::Path;
use std::path::PathBuf;

use url::Url;
use thiserror::Error;
use reqwest::Client;
use futures::TryFutureExt;
use serde::{Serialize, Deserialize};

use super::VERSION_TAG_MAJOR_MINOR;

pub const INDEXERS_DIR: &str = "/config/indexers/";
pub const INDEXER_SPECIFICATIONS_DIR: &str = "/config/indexers/specifications/";

/////////////////////////////////////////////////////
// IndexerSpecification
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerSpecification {
    pub name: String,
    pub url: Url,
    pub mirrors: Vec<Url>,

    pub uses_cloudflare: bool,

    pub download: DownloadMethod,
}

/////////////////////////////////////////////////////
// Indexer
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    pub name: String,
    pub url: Url,
    pub uses_cloudflare: bool,

    pub download: DownloadMethod,

    pub based_on: PathBuf,
}

/////////////////////////////////////////////////////
// Specifications
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

/////////////////////////////////////////////////////
// IndexerType
/////////////////////////////////////////////////////
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

/////////////////////////////////////////////////////
// GetSpecificationError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum GetSpecificationError {
    #[error("Failed to create {INDEXER_SPECIFICATIONS_DIR} directory with error: {0}")]
    FailedToCreateIndexersSpecificationDirectory(std::io::Error),
    #[error("Failed to build HTTP client for retrieving indexers with error: {0}")]
    FailedToBuildHTTPClient(reqwest::Error),
    #[error("Unable to find indexers directory for version '{0}' in the FMHY-Indexers repository, error: {1}.")]
    UnableToFindIndexersForVersion(&'static str, String),
    #[error("Unable to parse GitHub API response due to error: {0}.")]
    UnableToParseGHListing(reqwest::Error),
    #[error("Unable to download indexer specification from \"{0}\" due to error: {1}")]
    UnableToDownloadIndexer(Url, reqwest::Error),
    #[error("Unable to write indexer specification \"{0}\" to \"{1}\" due to error: {2}")]
    UnableToWriteIndexer(String, PathBuf, std::io::Error),
}

pub async fn parse_indexer_specification_from_file(file: &Path) -> Result<IndexerSpecification, ParseIndexError> {
    if !file.exists() {
        return Err(ParseIndexError::FileDoesntExist(file.to_path_buf()));
    }

    let contents = tokio::fs::read_to_string(file).map_err(ParseIndexError::UnableToReadFile).await?;

    let json_body: IndexerSpecification = serde_json::from_str(contents.as_str()).map_err(|error| ParseIndexError::UnableToParseJson {
        json: contents,
        error: error,
    })?;

    Ok(json_body)
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

pub async fn load_indexers() -> Vec<Indexer> {
    let indexer_paths = std::fs::read_dir(INDEXERS_DIR);
    let Ok(indexer_paths) = indexer_paths else {
        return Vec::new();
    };

    let mut indexers: Vec<Indexer> = Vec::new();

    for indexer_path in indexer_paths {
        match indexer_path {
            Ok(entry) => {
                if let Ok(file_type) = entry.file_type()
                    && (!file_type.is_dir() && file_type.is_file())
                {
                    trace!("Found indexer path: {}", entry.path().display());

                    match parse_indexer_from_file(entry.path().as_path()).await {
                        Ok(indexer) => indexers.push(indexer),
                        Err(error) => error!("Failed to parse indexer \"{}\" with error: {}", entry.path().display(), error),
                    }
                }
            },
            Err(error) => warning!("Error reading indexer folder entry: {}", error),
        }
    }

    indexers
}

pub async fn load_indexer_specifications() -> Vec<IndexerSpecification> {
    let indexer_specification_paths = std::fs::read_dir(INDEXER_SPECIFICATIONS_DIR);
    let Ok(indexer_specification_paths) = indexer_specification_paths else {
        return Vec::new();
    };

    let mut specifications: Vec<IndexerSpecification> = Vec::new();

    for indexer_path in indexer_specification_paths {
        match indexer_path {
            Ok(entry) => {
                if let Ok(file_type) = entry.file_type()
                    && (!file_type.is_dir() && file_type.is_file())
                {
                    trace!("Found indexer path: {}", entry.path().display());

                    match parse_indexer_specification_from_file(entry.path().as_path()).await {
                        Ok(specification) => specifications.push(specification),
                        Err(error) => error!("Failed to parse indexer specification \"{}\" with error: {}", entry.path().display(), error),
                    }
                }
            },
            Err(error) => warning!("Error reading indexer folder entry: {}", error),
        }
    }

    specifications
}

pub async fn get_new_specifications() -> Result<(), GetSpecificationError> {
    const REPO_API: &str = "https://api.github.com/repos/VOD-Downloaders/FMHY-Indexers";

    let indexer_specifications_dir = PathBuf::from(INDEXER_SPECIFICATIONS_DIR);
    if !indexer_specifications_dir.exists() {
        tokio::fs::create_dir(indexer_specifications_dir.as_path())
            .await
            .map_err(GetSpecificationError::FailedToCreateIndexersSpecificationDirectory)?;
    }

    trace!("Retrieving latest indexers for version '{}' from https://github.com/VOD-Downloaders/FMHY-Indexers...", VERSION_TAG_MAJOR_MINOR);

    // Create HTTP client
    let client = Client::builder()
        .user_agent(concat!("FMHY-Downloader/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(GetSpecificationError::FailedToBuildHTTPClient)?;

    // Retrieve files in version directory
    let response = client
        .get(format!("{}/contents/{}", REPO_API, VERSION_TAG_MAJOR_MINOR))
        .send()
        .await
        .map_err(|error| GetSpecificationError::UnableToFindIndexersForVersion(VERSION_TAG_MAJOR_MINOR, error.to_string()))?;

    if !response.status().is_success() {
        return Err(GetSpecificationError::UnableToFindIndexersForVersion(VERSION_TAG_MAJOR_MINOR, response.status().to_string()));
    }

    let listing: Vec<serde_json::Value> = response.json().await.map_err(GetSpecificationError::UnableToParseGHListing)?;

    // Loop through files
    for entry in &listing {
        // Skip non-JSON entries
        let name = match entry["name"].as_str() {
            Some(n) if n.ends_with(".json") => n,
            _ => continue,
        };

        let download_url = match entry["download_url"].as_str() {
            Some(url) => url,
            None => {
                warning!("No download url for '{}', skipping...", name);
                continue;
            },
        };
        let download_url = match Url::parse(download_url) {
            Ok(url) => url,
            Err(error) => {
                warning!("Invalid download url (\"{}\") for {}, skipping... Error: {}", download_url, name, error);
                continue;
            },
        };

        let bytes = client
            .get(download_url.as_str())
            .send()
            .await
            .map_err(|error| GetSpecificationError::UnableToDownloadIndexer(download_url.clone(), error))?
            .bytes()
            .await
            .map_err(|error| GetSpecificationError::UnableToDownloadIndexer(download_url.clone(), error))?;

        let path = indexer_specifications_dir.join(name);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| GetSpecificationError::UnableToWriteIndexer(name.to_string(), path.clone(), error))?;
    }

    Ok(())
}

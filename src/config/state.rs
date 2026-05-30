use std::error::Error;
use std::path::PathBuf;

use thiserror::Error;
use url::Url;
use futures::TryFutureExt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use serde::{Serialize, Deserialize};

use super::VERSION_TAG_MAJOR_MINOR;
use super::Indexer;
use super::parse_indexer_from_file;

/////////////////////////////////////////////////////
// StateError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Unable to read state.json with error: {0}")]
    UnableToReadFile(std::io::Error),
    #[error("Unable to parse json \"{json}\" due to error: {error}")]
    UnableToParseJson { json: String, error: serde_json::Error },
    #[error("Unable to read /config/indexers directory due to error: {0}")]
    UnableToReadIndexersDir(std::io::Error),

    #[error("Failed to create /config/indexers directory with error: {0}")]
    FailedToCreateIndexersDirectory(std::io::Error),
    #[error("Failed to build HTTP client for retrieving indexers with error: {0}")]
    FailedToBuildHTTPClient(reqwest::Error),
    #[error("Unable to find indexers directory for version '{0}' in the FMHY-Indexers repository, error: {1}.")]
    UnableToFindIndexersForVersion(&'static str, String),
    #[error("Unable to parse GitHub API response due to error: {0}.")]
    UnableToParseGHListing(reqwest::Error),
    #[error("Unable to download indexer from \"{0}\" due to error: {1}")]
    UnableToDownloadIndexer(Url, reqwest::Error),
    #[error("Unable to write indexer \"{0}\" to \"{1}\" due to error: {2}")]
    UnableToWriteIndexer(String, PathBuf, std::io::Error),

    #[error("Unable to open state.json for writing.")]
    UnableToOpen(std::io::Error),
    #[error("Unable to convert object to json string, error: {0}")]
    UnableToConvert(serde_json::Error),
    #[error("Unable to write json to state.json with error: {0}")]
    UnableToWrite(std::io::Error),
}

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct StateBody {
    todo: i8,
}

impl StateBody {
    fn from(_state: &State) -> Self {
        Self { todo: 0 }
    }
}

#[derive(Debug)]
pub struct State {
    pub indexers: Vec<Indexer>,
}

impl State {
    const FILE: &str = "/config/state.json";
    const INDEXERS_DIR: &str = "/config/indexers/";

    pub async fn get_indexers() -> Result<Vec<Indexer>, StateError> {
        const REPO_API: &str = "https://api.github.com/repos/VOD-Downloaders/FMHY-Indexers";

        let indexers_dir = PathBuf::from(Self::INDEXERS_DIR);
        if !indexers_dir.exists() {
            tokio::fs::create_dir(indexers_dir.as_path())
                .await
                .map_err(StateError::FailedToCreateIndexersDirectory)?;
        }

        trace!("Retrieving latest indexers for version '{}' from https://github.com/VOD-Downloaders/FMHY-Indexers...", VERSION_TAG_MAJOR_MINOR);

        // Create HTTP client
        let client = Client::builder()
            .user_agent(concat!("FMHY-Downloader/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(StateError::FailedToBuildHTTPClient)?;

        // Retrieve files in version directory
        let response = client
            .get(format!("{}/contents/{}", REPO_API, VERSION_TAG_MAJOR_MINOR))
            .send()
            .await
            .map_err(|error| StateError::UnableToFindIndexersForVersion(VERSION_TAG_MAJOR_MINOR, error.to_string()))?;

        if !response.status().is_success() {
            return Err(StateError::UnableToFindIndexersForVersion(VERSION_TAG_MAJOR_MINOR, response.status().to_string()));
        }

        let listing: Vec<serde_json::Value> = response.json().await.map_err(StateError::UnableToParseGHListing)?;

        // Loop through files
        let mut indexers: Vec<Indexer> = Vec::new();

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
                .map_err(|error| StateError::UnableToDownloadIndexer(download_url.clone(), error))?
                .bytes()
                .await
                .map_err(|error| StateError::UnableToDownloadIndexer(download_url.clone(), error))?;

            let path = indexers_dir.join(name);
            tokio::fs::write(&path, &bytes)
                .await
                .map_err(|error| StateError::UnableToWriteIndexer(name.to_string(), path.clone(), error))?;

            match parse_indexer_from_file(path.as_path()).await {
                Ok(indexer) => {
                    trace!("Parsed indexer '{}'", name);
                    indexers.push(indexer);
                },
                Err(error) => error!("Failed to parse indexer '{}': {}", name, error),
            }
        }

        Ok(indexers)
    }

    pub async fn make_default_state() -> Result<Self, StateError> {
        let state = State {
            indexers: Self::get_indexers().await?,
        };

        // Write to disk
        state.write().await?;

        Ok(state)
    }

    pub async fn retrieve() -> Result<Self, StateError> {
        trace!("Retrieving State from state.json...");

        // Retrieve json body
        let path: PathBuf = PathBuf::from(Self::FILE);
        if !path.exists() {
            warning!("Failed to retrieve state.json, starting up with empty configuration.");
            let state = Self::make_default_state().await?;
            return Ok(state);
        }

        let contents = tokio::fs::read_to_string(Self::FILE).map_err(StateError::UnableToReadFile).await?;

        let json_body: StateBody = serde_json::from_str(contents.as_str()).map_err(|error| StateError::UnableToParseJson {
            json: contents,
            error: error,
        })?;

        // Retrieve indexers
        let indexer_paths = std::fs::read_dir(Self::INDEXERS_DIR).map_err(StateError::UnableToReadIndexersDir)?;
        let mut indexers: Vec<Indexer> = Vec::new();

        for indexer_path in indexer_paths {
            match indexer_path {
                Ok(entry) => {
                    trace!("Found indexer path: {}", entry.path().display());

                    match parse_indexer_from_file(entry.path().as_path()).await {
                        Ok(indexer) => indexers.push(indexer),
                        Err(error) => error!("Failed to parse indexer \"{}\" with error: {}", entry.path().display(), error),
                    }
                },
                Err(error) => warning!("Error reading indexer folder entry: {}", error),
            }
        }

        // New indexers
        // TODO: ...

        Ok(Self { indexers: indexers })
    }

    pub async fn write(&self) -> Result<(), StateError> {
        trace!("Opening state.json for writing...");

        let mut file = OpenOptions::new().create(true).append(true).open(Self::FILE).await.map_err(|error| {
            trace!("Failed to open \"{}\", error: {:?}, source: {:?}", Self::FILE, error, error.source());

            StateError::UnableToOpen(error)
        })?;

        let json = serde_json::to_string(&StateBody::from(self)).map_err(StateError::UnableToConvert)?;

        file.write_all(json.as_bytes()).await.map_err(StateError::UnableToWrite)?;

        Ok(())
    }
}

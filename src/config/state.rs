use std::error::Error;
use std::path::PathBuf;

use thiserror::Error;
use futures::TryFutureExt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

use super::Indexer;
use super::parse_indexer_from_file;

/////////////////////////////////////////////////////
// StateError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum StateError {
    #[error("No state.json file found")]
    NoFile,
    #[error("Unable to read state.json with error: {0}")]
    UnableToReadFile(std::io::Error),
    #[error("Unable to parse json \"{json}\" due to error: {error}")]
    UnableToParseJson { json: String, error: serde_json::Error },
    #[error("Unable to read /config/indexers directory due to error: {0}")]
    UnableToReadIndexersDir(std::io::Error),

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
    indexers_commit: String,
}

impl StateBody {
    fn from(state: &State) -> Self {
        Self {
            indexers_commit: state.indexers_commit.clone(),
        }
    }
}

#[derive(Debug)]
pub struct State {
    indexers_commit: String,
    indexers: Vec<Indexer>,
}

impl State {
    const FILE: &str = "state.json";

    pub async fn get_indexers() -> Result<(), StateError> {
        // TODO: ...
        Ok(())
    }

    pub async fn make_default_state() -> Result<Self, StateError> {
        let mut state = State {
            indexers_commit: "".to_string(),
            indexers: Vec::new(),
        };
        // Indexers

        // State

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
        let indexer_paths = std::fs::read_dir("./indexers").map_err(StateError::UnableToReadIndexersDir)?;
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

        Ok(Self {
            indexers_commit: json_body.indexers_commit,
            indexers: indexers,
        })
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

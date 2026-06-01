use std::error::Error;
use std::path::PathBuf;

use thiserror::Error;
use futures::TryFutureExt;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

use crate::config::get_new_specifications;
use crate::config::load_indexers;

use super::Indexer;
use super::GetSpecificationError;

/////////////////////////////////////////////////////
// StateError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Unable to read state.json with error: {0}")]
    UnableToReadFile(std::io::Error),
    #[error("Unable to parse json \"{json}\" due to error: {error}")]
    UnableToParseJson { json: String, error: serde_json::Error },

    #[error("{0}")]
    UnableToRetrieveSpecifications(GetSpecificationError),

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

    pub async fn make_default_state() -> Result<Self, StateError> {
        get_new_specifications().await.map_err(StateError::UnableToRetrieveSpecifications)?; // Get latest indexers

        let state = State { indexers: Vec::new() };
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

        let _json_body: StateBody = serde_json::from_str(contents.as_str()).map_err(|error| StateError::UnableToParseJson {
            json: contents,
            error: error,
        })?;

        let indexers = load_indexers().await;

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

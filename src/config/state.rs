use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use futures::FutureExt;
use thiserror::Error;
use futures::TryFutureExt;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// StateError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum StateError {
    #[error("No state.json file found")]
    NoFile,
    #[error("Unable to read state.json with error: {0}")]
    UnableToReadFile(std::io::Error),
    #[error("")]
    UnableToParseJson { json: String, error: serde_json::Error },

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
pub struct State {
    indexers_commit: String,
}

impl State {
    const FILE: &str = "state.json";

    pub async fn retrieve() -> Result<State, StateError> {
        trace!("Retrieving State from state.json...");

        let path: PathBuf = PathBuf::from(Self::FILE);
        if !path.exists() {
            return Err(StateError::NoFile);
        }

        let contents = tokio::fs::read_to_string(Self::FILE).map_err(StateError::UnableToReadFile).await?;

        let object: State = serde_json::from_str(contents.as_str()).map_err(|error| StateError::UnableToParseJson {
            json: contents,
            error: error,
        })?;

        Ok(object)
    }

    pub async fn write(&self) -> Result<(), StateError> {
        trace!("Opening state.json for writing...");

        let mut file = OpenOptions::new().create(true).append(true).open(Self::FILE).await.map_err(|error| {
            trace!("Failed to open \"{}\", error: {:?}, source: {:?}", Self::FILE, error, error.source());

            StateError::UnableToOpen(error)
        })?;

        let json = serde_json::to_string(self).map_err(StateError::UnableToConvert)?;

        file.write_all(json.as_bytes()).await.map_err(StateError::UnableToWrite)?;

        Ok(())
    }
}

use std::path::PathBuf;

use crate::error::Error;

pub struct State {
    path: PathBuf,
    state_inner: StateInner,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StateInner {}

impl State {
    pub async fn load_from_path(path: PathBuf) -> Result<Self, Error> {
        tokio::fs::read_to_string(&path)
            .await
            .map_err(Error::ReadingState)
            .and_then(|text| toml::from_str(&text).map_err(Error::ParsingState))
            .map(|state_inner| State { path, state_inner })
    }
}

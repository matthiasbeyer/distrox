use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use crate::error::Error;

pub struct Configuration {
    path: PathBuf,
    config: Config,
}

impl Configuration {
    pub async fn load_from_path(path: PathBuf) -> Result<Self, Error> {
        tokio::fs::read_to_string(&path)
            .await
            .map_err(Error::ReadingConfig)
            .and_then(|text| toml::from_str(&text).map_err(Error::ParsingConfig))
            .map(|config| Configuration { path, config })
    }

    pub async fn save(&self) -> Result<(), Error> {
        let config = toml::to_string(&self.config).map_err(Error::SerializingConfig)?;

        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.path)
            .await
            .map_err(|source| Error::OpenConfigFile {
                path: self.path.to_path_buf(),
                source,
            })?
            .write_all(config.as_bytes())
            .await
            .map_err(|source| Error::WritingConfig {
                path: self.path.to_path_buf(),
                source,
            })
    }

    pub fn network(&self) -> &Network {
        &self.config.network
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    network: Network,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Network {
    storage_path: PathBuf,
    bootstrap_nodes: Vec<Multiaddr>,
    listening_addrs: Vec<Multiaddr>,
}

impl Network {
    pub(crate) fn storage_path(&self) -> &PathBuf {
        &self.storage_path
    }

    pub(crate) fn bootstrap_nodes(&self) -> &[Multiaddr] {
        self.bootstrap_nodes.as_ref()
    }

    pub(crate) fn listening_addrs(&self) -> &[Multiaddr] {
        self.listening_addrs.as_ref()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Multiaddr(String);

impl TryFrom<Multiaddr> for libp2p::Multiaddr {
    type Error = Error;

    fn try_from(m: Multiaddr) -> Result<libp2p::Multiaddr, Self::Error> {
        m.0.parse().map_err(|source| Error::ParseMultiAddr {
            addr: m.0.clone(),
            source,
        })
    }
}

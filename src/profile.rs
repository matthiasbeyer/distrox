use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use tokio::io::AsyncReadExt;

use crate::client::Client;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

#[derive(Debug)]
pub struct Profile {
    client: Client,
}

impl Profile {
    pub async fn create(state_dir: &Path, name: &str, config: Config) -> Result<Self> {
        let bootstrap = vec![]; // TODO
        let mdns = false; // TODO
        let keypair = ipfs::Keypair::generate_ed25519();
        Self::write_to_statedir(state_dir, name, &keypair).await?;

        let options = ipfs::IpfsOptions {
            ipfs_path: Self::ipfs_path(state_dir, name).await?,
            keypair,
            bootstrap,
            mdns,
            kad_protocol: None,
            listening_addrs: vec![],
            span: Some(tracing::trace_span!("distrox-ipfs")),
        };

        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<ipfs::Types>::new(options)
            .start()
            .await?;
        tokio::task::spawn(fut);
        Ok(Self::new(ipfs, config))
    }

    async fn new_inmemory(config: Config) -> Result<Self> {
        let mut opts = ipfs::IpfsOptions::inmemory_with_generated_keys();
        opts.mdns = false;
        let (ipfs, fut): (ipfs::Ipfs<ipfs::Types>, _) = ipfs::UninitializedIpfs::new(opts).start().await.unwrap();
        tokio::task::spawn(fut);
        Ok(Self::new(ipfs, config))
    }

    fn new(ipfs: IpfsClient, config: Config) -> Self {
        Profile { client: Client::new(ipfs, config) }
    }

    async fn write_to_statedir(_state_dir: &Path, _name: &str, _keypair: &ipfs::Keypair) -> Result<()> {
        unimplemented!()
    }

    async fn ipfs_path(state_dir: &Path, name: &str) -> Result<PathBuf> {
        let path = state_dir.join(name).join("ipfs");
        tokio::fs::create_dir_all(&path).await?;
        Ok(path)
    }

    pub fn config_path(name: &str) -> String {
        format!("distrox-{}", name)
    }

    pub fn config_file_path(name: &str) -> Result<PathBuf> {
        xdg::BaseDirectories::with_prefix("distrox")
            .map_err(anyhow::Error::from)
            .and_then(|dirs| {
                let name = Self::config_path(name);
                dirs.place_config_file(name)
                    .map_err(anyhow::Error::from)
            })
    }

    /// Load the Profile from disk and ensure the keys exist in IPFS
    pub async fn load_from_filesystem(_name: &str, _client: &Client) -> Result<Option<Self>> {
        unimplemented!()
    }

    async fn load_from_reader<R: AsyncReadExt + std::marker::Unpin>(_r: R, _name: &str, _client: &Client) -> Result<Option<Self>> {
        unimplemented!()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;

    #[tokio::test]
    async fn test_create_profile() {
        let _ = env_logger::try_init();
        let profile = Profile::new_inmemory(Config::default()).await;
        assert!(profile.is_ok());
    }

}

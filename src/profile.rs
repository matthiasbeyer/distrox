use std::path::PathBuf;

use anyhow::Result;
use ipfs_api_backend_hyper::IpfsApi;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

use crate::client::Client;
use crate::cid::Cid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    key_name: String,
    key_id: String,
}

impl Profile {
    pub async fn create(name: &str, client: &Client) -> Result<Self> {
        let key = client.ipfs.key_gen(name, ipfs_api_backend_hyper::KeyType::Ed25519, 64).await?;

        Ok(Profile {
            key_name: key.name,
            key_id: key.id
        })
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

    /// Store the Profile on disk
    pub async fn write_to_filesystem(&self) -> Result<()> {
        let config_path = Self::config_file_path(&self.key_name)?;

        let mut config_file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)
            .await?;

        let config = serde_json::to_string(&self)?;
        config_file.write_all(config.as_bytes()).await?;
        config_file.sync_all().await?;
        Ok(())
    }

    /// Load the Profile from disk and ensure the keys exist in IPFS
    pub async fn load_from_filesystem(name: &str, client: &Client) -> Result<Option<Self>> {
        let config_path = Self::config_file_path(name)?;
        let file_reader = tokio::fs::OpenOptions::new()
            .read(true)
            .open(config_path)
            .await
            .map(tokio::io::BufReader::new)?;

        Self::load_from_reader(file_reader, name, client).await
    }

    async fn load_from_reader<R: AsyncReadExt + std::marker::Unpin>(mut r: R, name: &str, client: &Client) -> Result<Option<Self>> {
        let mut buf = String::new();
        let _ = r.read_to_string(&mut buf).await?;
        let config: Self = serde_json::from_str(&buf)?;

        client.ipfs
            .key_list()
            .await?
            .keys
            .into_iter()
            .find(|keypair| keypair.name == name)
            .map(|_| Ok(config))
            .transpose()
    }

    pub async fn publish(&self, client: &Client, cid: Cid) -> Result<()> {
        let path = format!("/ipfs/{}", cid.as_ref());
        let resolve = true;
        let lifetime = Some("10m");
        let ttl = None;

        let publish_response = client.ipfs
            .name_publish(&path, resolve, lifetime, ttl, Some(&self.key_name))
            .await?;

        log::debug!("Publish response = {{ name: {}, value: {} }}", publish_response.name, publish_response.value);
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;

    use ipfs_api_backend_hyper::TryFromUri;

    async fn mk_client() -> Client {
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        Client::new(ipfs, config)
    }

    macro_rules! run_test {
        ($name:ident, $client:ident, $test:block) => {
            $client.ipfs.key_rm($name).await;
            {
                $test
            }
            $client.ipfs.key_rm($name).await;
        }
    }

    #[tokio::test]
    async fn test_create_profile() {
        let _ = env_logger::try_init();
        let client = mk_client().await;
        let name = "test_create_profile";
        run_test!(name, client,
            {
                let p = Profile::create(name, &client).await;
                assert!(p.is_ok());
            }
        );
    }

}

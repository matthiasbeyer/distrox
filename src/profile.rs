use std::path::Path;
use std::path::PathBuf;
use std::convert::TryFrom;
use std::convert::TryInto;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::client::Client;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

#[derive(Debug)]
pub struct Profile {
    state: ProfileState,
    client: Client,
}

impl Profile {
    pub async fn create(state_dir: &StateDir, name: &str, config: Config) -> Result<Self> {
        let bootstrap = vec![]; // TODO
        let mdns = false; // TODO
        let keypair = ipfs::Keypair::generate_ed25519();

        let options = ipfs::IpfsOptions {
            ipfs_path: Self::ipfs_path(state_dir, name).await?,
            keypair,
            bootstrap,
            mdns,
            kad_protocol: None,
            listening_addrs: vec![],
            span: Some(tracing::trace_span!("distrox-ipfs")),
        };

        let keypair = options.keypair.clone();
        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<_>::new(options)
            .start()
            .await?;
        tokio::task::spawn(fut);
        Self::new(ipfs, config, name.to_string(), keypair).await
    }

    async fn new_inmemory(config: Config, name: &str) -> Result<Self> {
        let mut opts = ipfs::IpfsOptions::inmemory_with_generated_keys();
        opts.mdns = false;
        let keypair = opts.keypair.clone();
        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<_>::new(opts).start().await.unwrap();
        tokio::task::spawn(fut);
        Self::new(ipfs, config, format!("inmemory-{}", name), keypair).await
    }

    async fn new(ipfs: IpfsClient, config: Config, profile_name: String, keypair: libp2p::identity::Keypair) -> Result<Self> {
        let client = Client::new(ipfs, config);
        let profile_head = Self::post_hello_world(&client, &profile_name).await?;
        let state = ProfileState {
            profile_head,
            profile_name,
            keypair,
        };
        Ok(Profile { state, client })
    }

    async fn post_hello_world(client: &Client, name: &str) -> Result<cid::Cid> {
        let text = format!("Hello world, I am {}", name);
        client.post_text_node(vec![], text).await
    }

    async fn ipfs_path(state_dir: &StateDir, name: &str) -> Result<PathBuf> {
        let path = state_dir.ipfs();
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

    pub fn state_dir_path(name: &str) -> Result<StateDir> {
        xdg::BaseDirectories::with_prefix("distrox")
            .map_err(anyhow::Error::from)
            .and_then(|dirs| {
                dirs.create_state_directory(name)
                    .map(StateDir::from)
                    .map_err(anyhow::Error::from)
            })
    }

    pub async fn save(&self) -> Result<()> {
        let state_dir_path = Self::state_dir_path(&self.state.profile_name)?;
        ProfileStateSaveable::new(&self.state)?
            .save_to_disk(&state_dir_path)
            .await
    }

    pub async fn load(config: Config, name: &str) -> Result<Self> {
        let state_dir_path = Self::state_dir_path(name)?;
        let state: ProfileState = ProfileStateSaveable::load_from_disk(&state_dir_path)
            .await?
            .try_into()?;

        let bootstrap = vec![]; // TODO
        let mdns = false; // TODO
        let keypair = state.keypair.clone();

        let options = ipfs::IpfsOptions {
            ipfs_path: Self::ipfs_path(&state_dir_path, name).await?,
            keypair,
            bootstrap,
            mdns,
            kad_protocol: None,
            listening_addrs: vec![],
            span: Some(tracing::trace_span!("distrox-ipfs")),
        };

        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<_>::new(options)
            .start()
            .await?;
        tokio::task::spawn(fut);

        Ok(Profile {
            state,
            client: Client::new(ipfs, config),
        })
    }

}

#[derive(Debug)]
pub struct StateDir(PathBuf);

impl StateDir {
    pub fn ipfs(&self) -> PathBuf {
        self.0.join("ipfs")
    }

    pub fn profile_state(&self) -> PathBuf {
        self.0.join("profile_state")
    }

    pub fn display(&self) -> std::path::Display {
        self.0.display()
    }
}

impl From<PathBuf> for StateDir {
    fn from(p: PathBuf) -> Self {
        Self(p)
    }
}

#[derive(getset::Getters)]
pub struct ProfileState {
    #[getset(get = "pub")]
    profile_head: cid::Cid,

    #[getset(get = "pub")]
    profile_name: String,

    #[getset(get = "pub")]
    keypair: libp2p::identity::Keypair,
}

impl std::fmt::Debug for ProfileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileState {{ name = {}, head = {:?} }}", self.profile_name, self.profile_head)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, getset::Getters)]
struct ProfileStateSaveable {
    profile_head: Vec<u8>,
    profile_name: String,
    keypair: Vec<u8>,
}

impl ProfileStateSaveable {
    fn new(s: &ProfileState) -> Result<Self> {
        Ok(Self {
            profile_head: s.profile_head.to_bytes(),
            profile_name: s.profile_name.clone(),
            keypair: match s.keypair {
                libp2p::identity::Keypair::Ed25519(ref kp) => Vec::from(kp.encode()),
                _ => anyhow::bail!("Only keypair type ed25519 supported"),
            }
        })
    }

    pub async fn save_to_disk(&self, state_dir_path: &StateDir) -> Result<()> {
        let state_s = serde_json::to_string(&self)?;
        tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .open(&state_dir_path.profile_state())
            .await?
            .write_all(state_s.as_bytes())
            .await
            .map(|_| ())
            .map_err(anyhow::Error::from)
    }

    pub async fn load_from_disk(state_dir_path: &StateDir) -> Result<Self> {
        let reader = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&state_dir_path.profile_state())
            .await?
            .into_std()
            .await;

        serde_json::from_reader(reader).map_err(anyhow::Error::from)
    }

}

impl TryInto<ProfileState> for ProfileStateSaveable {
    type Error = anyhow::Error;

    fn try_into(mut self) -> Result<ProfileState> {
        Ok(ProfileState {
            profile_head: cid::Cid::try_from(self.profile_head)?,
            profile_name: self.profile_name,
            keypair: {
                let kp = libp2p::identity::ed25519::Keypair::decode(&mut self.keypair)?;
                libp2p::identity::Keypair::Ed25519(kp)
            },
        })
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

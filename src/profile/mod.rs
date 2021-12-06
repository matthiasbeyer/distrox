use std::path::PathBuf;
use std::convert::TryInto;

use anyhow::Context;
use anyhow::Result;

use crate::client::Client;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

mod state;
use state::*;

#[derive(Debug)]
pub struct Profile {
    state: ProfileState,
    client: Client,
}

impl Profile {
    pub async fn create(state_dir: &StateDir, name: &str, config: Config) -> Result<Self> {
        let bootstrap = vec![]; // TODO
        let mdns = true; // TODO
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
        opts.mdns = true;
        let keypair = opts.keypair.clone();
        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<_>::new(opts).start().await.unwrap();
        tokio::task::spawn(fut);
        Self::new(ipfs, config, format!("inmemory-{}", name), keypair).await
    }

    async fn new(ipfs: IpfsClient, config: Config, profile_name: String, keypair: libp2p::identity::Keypair) -> Result<Self> {
        let client = Client::new(ipfs, config);
        let profile_head = Self::post_hello_world(&client, &profile_name).await?;
        let state = ProfileState::new(profile_head, profile_name, keypair);
        Ok(Profile { state, client })
    }

    pub fn head(&self) -> &cid::Cid {
        self.state.profile_head()
    }

    pub async fn connect(&self, peer: ipfs::MultiaddrWithPeerId) -> Result<()> {
        self.client.connect(peer).await
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
        log::debug!("Getting state directory path");
        xdg::BaseDirectories::with_prefix("distrox")
            .context("Fetching 'distrox' XDG base directory")
            .map_err(anyhow::Error::from)
            .and_then(|dirs| {
                dirs.create_state_directory(name)
                    .map(StateDir::from)
                    .with_context(|| format!("Creating 'distrox' XDG state directory for '{}'", name))
                    .map_err(anyhow::Error::from)
            })
    }

    pub async fn save(&self) -> Result<()> {
        let state_dir_path = Self::state_dir_path(self.state.profile_name())?;
        log::trace!("Saving to {:?}", state_dir_path.display());
        ProfileStateSaveable::new(&self.state)
            .context("Serializing profile state")?
            .save_to_disk(&state_dir_path)
            .await
            .context("Saving state to disk")
            .map_err(anyhow::Error::from)
    }

    pub async fn load(config: Config, name: &str) -> Result<Self> {
        let state_dir_path = Self::state_dir_path(name)?;
        log::trace!("state_dir_path = {:?}", state_dir_path.display());
        let state: ProfileState = ProfileStateSaveable::load_from_disk(&state_dir_path)
            .await?
            .try_into()
            .context("Parsing profile state")?;
        log::debug!("Loading state finished");

        let bootstrap = vec![]; // TODO
        let mdns = true; // TODO
        let keypair = state.keypair().clone();

        log::debug!("Configuring IPFS backend");
        let options = ipfs::IpfsOptions {
            ipfs_path: Self::ipfs_path(&state_dir_path, name).await?,
            keypair,
            bootstrap,
            mdns,
            kad_protocol: None,
            listening_addrs: vec![],
            span: Some(tracing::trace_span!("distrox-ipfs")),
        };

        log::debug!("Starting IPFS backend");
        let (ipfs, fut): (ipfs::Ipfs<_>, _) = ipfs::UninitializedIpfs::<_>::new(options)
            .start()
            .await?;
        tokio::task::spawn(fut);

        log::debug!("Profile loading finished");
        Ok(Profile {
            state,
            client: Client::new(ipfs, config),
        })
    }

    pub async fn exit(self) -> Result<()> {
        self.client.exit().await
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;

    #[tokio::test]
    async fn test_create_profile() {
        let _ = env_logger::try_init();
        let profile = Profile::new_inmemory(Config::default(), "test-create-profile").await;
        assert!(profile.is_ok());
        let exit = profile.unwrap().exit().await;
        assert!(exit.is_ok(), "Not cleanly exited: {:?}", exit);
    }

    #[tokio::test]
    async fn test_create_profile_and_helloworld() {
        let _ = env_logger::try_init();
        let profile = Profile::new_inmemory(Config::default(), "test-create-profile-and-helloworld").await;
        assert!(profile.is_ok());
        let profile = profile.unwrap();
        let head = profile.head();
        let exp_cid = cid::Cid::try_from("bafyreie4haukbqj7u6vogjfvaxbwg73b7bzi7nqxbnkvv77dvwcqg5wtpe").unwrap();
        assert_eq!(*head, exp_cid, "{} != {}", *head, exp_cid);
        let exit = profile.exit().await;
        assert!(exit.is_ok(), "Not cleanly exited: {:?}", exit);
    }

}

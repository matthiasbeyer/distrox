use std::path::PathBuf;
use std::convert::TryFrom;
use std::convert::TryInto;

use anyhow::Context;
use anyhow::Result;
use tokio::io::AsyncWriteExt;

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
    profile_head: Option<cid::Cid>,

    #[getset(get = "pub")]
    profile_name: String,

    #[getset(get = "pub")]
    keypair: libp2p::identity::Keypair,
}

impl ProfileState {
    pub(super) fn new(profile_name: String, keypair: libp2p::identity::Keypair) -> Self {
        Self {
            profile_head: None,
            profile_name,
            keypair
        }
    }
}

impl std::fmt::Debug for ProfileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileState {{ name = {}, head = {:?} }}", self.profile_name, self.profile_head)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, getset::Getters)]
pub(super) struct ProfileStateSaveable {
    profile_head: Option<Vec<u8>>,
    profile_name: String,
    keypair: Vec<u8>,
}

impl ProfileStateSaveable {
    pub(super) fn new(s: &ProfileState) -> Result<Self> {
        Ok(Self {
            profile_head: s.profile_head.clone().map(|v| v.to_bytes()),
            profile_name: s.profile_name.clone(),
            keypair: match s.keypair {
                libp2p::identity::Keypair::Ed25519(ref kp) => Vec::from(kp.encode()),
                _ => anyhow::bail!("Only keypair type ed25519 supported"),
            }
        })
    }

    pub async fn save_to_disk(&self, state_dir_path: &StateDir) -> Result<()> {
        let state_s = serde_json::to_string(&self).context("Serializing state")?;
        tokio::fs::OpenOptions::new()
            .create_new(false) // do not _always_ create a new file
            .create(true)
            .truncate(true)
            .write(true)
            .open(&state_dir_path.profile_state())
            .await
            .with_context(|| format!("Opening {}", state_dir_path.profile_state().display()))?
            .write_all(state_s.as_bytes())
            .await
            .map(|_| ())
            .with_context(|| format!("Writing to {}", state_dir_path.profile_state().display()))
            .map_err(anyhow::Error::from)
    }

    pub async fn load_from_disk(state_dir_path: &StateDir) -> Result<Self> {
        log::trace!("Loading from disk: {:?}", state_dir_path.profile_state().display());
        let reader = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&state_dir_path.profile_state())
            .await
            .context("Opening state file")?
            .into_std()
            .await;

        log::trace!("Parsing state file");
        serde_json::from_reader(reader)
            .context("Parsing state file")
            .map_err(anyhow::Error::from)
    }

}

impl TryInto<ProfileState> for ProfileStateSaveable {
    type Error = anyhow::Error;

    fn try_into(mut self) -> Result<ProfileState> {
        Ok(ProfileState {
            profile_head: self.profile_head.map(|h| cid::Cid::try_from(h)).transpose()?,
            profile_name: self.profile_name,
            keypair: {
                let kp = libp2p::identity::ed25519::Keypair::decode(&mut self.keypair)?;
                libp2p::identity::Keypair::Ed25519(kp)
            },
        })
    }
}



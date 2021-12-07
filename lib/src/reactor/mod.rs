use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;

mod gossip;
mod device;
mod account;

/// Reactor type, for running the application logic
///
/// The Reactor runs the whole application logic, that is syncing with other devices, fetching and
/// keeping profile updates of other accounts, communication on the gossipsub topics... etc
#[derive(Debug)]
pub struct Reactor {
    profile: Arc<RwLock<Profile>>,
}

impl Reactor {
    pub fn new(profile: Profile) -> Self {
        Reactor {
            profile: Arc::new(RwLock::new(profile)),
        }
    }

    pub async fn head(&self) -> Option<cid::Cid> {
        self.profile.read().await.head().map(cid::Cid::clone)
    }

    pub async fn connect(&self, peer: ipfs::MultiaddrWithPeerId) -> Result<()> {
        self.profile.read().await.connect(peer).await
    }

    pub fn profile(&self) -> Arc<RwLock<Profile>> {
        self.profile.clone()
    }

    pub async fn exit(self) -> Result<()> {
        let mut inner = self.profile;
        loop {
            match Arc::try_unwrap(inner) {
                Err(arc) => inner = arc,
                Ok(inner) => return inner.into_inner().exit().await,
            }
        }
    }

    /// Run the reactor
    ///
    /// Starts all inner functionality and exposes things
    ///
    /// # Return
    ///
    /// Return types are WIP, as this must return "running" objects that can be communicated with
    pub async fn run(self) -> Result<()> {
        unimplemented!()
    }

}

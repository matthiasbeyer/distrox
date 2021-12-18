//! Low-level module for gossip'ing code
//!
//! This module implements the low-level gossiping functionality that other modules use to
//! implement actual behaviours on
//!

use std::sync::Arc;

use anyhow::Result;
use futures::Stream;
use futures::StreamExt;
use tokio::sync::RwLock;

use crate::profile::Profile;
use crate::gossip::GossipMessage;

#[derive(Debug)]
pub struct GossipHandler<Strategy = LogStrategy>
    where Strategy: GossipHandlingStrategy + Sync + Send
{
    profile: Arc<Profile>,
    strategy: std::marker::PhantomData<Strategy>,
}

impl<Strat> GossipHandler<Strat>
    where Strat: GossipHandlingStrategy + Sync + Send
{
    pub fn new(profile: Arc<Profile>) -> Self {
        Self {
            profile,
            strategy: std::marker::PhantomData,
        }
    }

    pub fn run<S>(self, input: S) -> impl Stream<Item = (GossipMessage, Result<()>)>
        where S: Stream<Item = (ipfs::PeerId, GossipMessage)>
    {
        input.then(move |(source, msg)| {
            let pr = self.profile.clone();
            async move {
                log::trace!("Received gossip message from {}: {:?}", source, msg);
                let res = Strat::handle_gossip_message(pr.clone(), &source, &msg).await;
                (msg, res)
            }
        })
    }
}

#[async_trait::async_trait]
pub trait GossipHandlingStrategy: Sync + Send {
    async fn handle_gossip_message(profile: Arc<Profile>, source: &ipfs::PeerId, msg: &GossipMessage) -> Result<()>;
}

pub struct LogStrategy;

#[async_trait::async_trait]
impl GossipHandlingStrategy for LogStrategy {
    async fn handle_gossip_message(_profile: Arc<Profile>, source: &ipfs::PeerId, msg: &GossipMessage) -> Result<()> {
        use std::convert::TryFrom;
        use std::ops::Deref;

        match msg {
            GossipMessage::CurrentProfileState { peer_id, cid } => {
                let peer_id = ipfs::PeerId::from_bytes(peer_id);
                let cid = cid::Cid::try_from(cid.deref());

                log::trace!("{:?} told me that {:?} is at {:?}", source, peer_id, cid);
            }
        }

        Ok(())
    }
}

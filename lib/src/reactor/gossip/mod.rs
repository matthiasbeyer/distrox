//! Low-level module for gossip'ing code
//!
//! This module implements the low-level gossiping functionality that other modules use to
//! implement actual behaviours on
//!

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;
use crate::reactor::Reactor;
use crate::reactor::ReactorReply;
use crate::reactor::ReactorRequest;
use crate::reactor::ctrl::ReactorReceiver;
use crate::reactor::ctrl::ReactorSender;
use crate::reactor::ctrl::ReplyChannel;

mod ctrl;
pub use ctrl::GossipRequest;
pub use ctrl::GossipReply;

mod msg;
pub use msg::GossipMessage;

#[derive(Debug)]
pub struct GossipReactor {
    inner: Reactor<GossipRequest, GossipReply>,
    gossip_topic_name: String,
}


impl GossipReactor {
    pub fn new(profile: Arc<RwLock<Profile>>, gossip_topic_name: String) -> (Self, ReactorSender<GossipRequest, GossipReply>) {
        let (inner, sender) = Reactor::<GossipRequest, GossipReply>::new(profile);
        let reactor = Self {
            inner,
            gossip_topic_name,
        };

        (reactor, sender)
    }

    pub async fn receive_next_message(&mut self) -> Option<(ReactorRequest<GossipRequest>, ReplyChannel<GossipReply>)> {
        self.inner.receive_next_message().await
    }

    fn send_gossip_reply(channel: ReplyChannel<GossipReply>, reply: GossipReply) -> Result<()> {
        if let Err(_) = channel.send(ReactorReply::Custom(reply)) {
            anyhow::bail!("Failed to send GossipReply::NoHead)")
        }

        Ok(())
    }

    pub(super) async fn process_reactor_message(&mut self, request: (ReactorRequest<GossipRequest>, ReplyChannel<GossipReply>)) -> Result<()> {
        match self.inner.process_reactor_message(request).await? {
            None => Ok(()),
            Some((GossipRequest::Ping, reply_channel)) => {
                if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::Pong)) {
                    anyhow::bail!("Failed sening PONG reply")
                }

                Ok(())
            },

            Some((GossipRequest::PublishMe, reply_channel)) => self.publish_me(reply_channel).await
        }
    }

    async fn publish_me(&self, reply_channel: ReplyChannel<GossipReply>) -> Result<()> {
        let profile = self.inner.profile();
        let profile = profile.read().await;

        let head = profile.head();
        if head.is_none() {
            Self::send_gossip_reply(reply_channel, GossipReply::NoHead)?;
            return Ok(())
        }
        let head = head.unwrap().to_bytes();

        let own_id = match profile.client().own_id().await {
            Ok(id) => id,
            Err(e) => {
                Self::send_gossip_reply(reply_channel, GossipReply::PublishMeResult(Err(e)))?;
                return Ok(()) // TODO: abort operation here for now, maybe not the best idea
            }
        };

        let publish_res = profile
            .client()
            .ipfs
            .pubsub_publish(self.gossip_topic_name.clone(), GossipMessage::CurrentProfileState {
                peer_id: own_id.to_bytes(),
                cid: head
            }.into_bytes()?)
            .await;

        match publish_res {
            Ok(()) => Self::send_gossip_reply(reply_channel, GossipReply::PublishMeResult(Ok(()))),
            Err(e) => Self::send_gossip_reply(reply_channel, GossipReply::PublishMeResult(Err(e))),
        }
    }

    async fn handle_gossip_message(&self, msg: Arc<ipfs::PubsubMessage>) -> Result<()> {
        use std::convert::TryFrom;

        match serde_json::from_slice(&msg.data) {
            Err(e) => log::trace!("Failed to deserialize gossip message from {}", msg.source),
            Ok(GossipMessage::CurrentProfileState { peer_id, cid }) => {
                let peer_id = ipfs::PeerId::from_bytes(&peer_id);
                let cid = cid::Cid::try_from(&*cid);
                log::trace!("Peer {:?} is at {:?}", peer_id, cid);

                // TODO start dispatched node chain fetching
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        use futures::stream::StreamExt;

        let mut subscription_stream = self.inner.profile()
            .read()
            .await
            .client()
            .ipfs
            .pubsub_subscribe(self.gossip_topic_name.clone())
            .await?;

        loop {
            tokio::select! {
                next_control_msg = self.receive_next_message() => {
                    match next_control_msg {
                        None => break,
                        Some(tpl) => self.process_reactor_message(tpl).await?,
                    }
                }

                next_gossip_message = subscription_stream.next() => {
                    if let Some(next_gossip_message) = next_gossip_message {
                        self.handle_gossip_message(next_gossip_message).await?;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}


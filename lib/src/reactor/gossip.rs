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


#[derive(Debug)]
pub struct GossipReactor {
    inner: Reactor<GossipRequest, GossipReply>,
    gossip_topic_name: String,
}

#[derive(Debug)]
pub enum GossipRequest {
    Ping,
    PublishMe,
}

#[derive(Debug)]
pub enum GossipReply {
    Pong,
    NoHead,
    PublishMeResult(Result<()>),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GossipMessage {
    CurrentProfileState {
        peer_id: Vec<u8>,
        cid: Vec<u8>,
    },
}

impl GossipMessage {
    fn into_bytes(self) -> Result<Vec<u8>> {
        serde_json::to_string(&self)
            .map(String::into_bytes)
            .map_err(anyhow::Error::from)
    }
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

    pub(super) async fn process_reactor_message(&mut self, request: (ReactorRequest<GossipRequest>, ReplyChannel<GossipReply>)) -> Result<()> {
        match self.inner.process_reactor_message(request).await? {
            None => Ok(()),
            Some((GossipRequest::Ping, reply_channel)) => {
                if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::Pong)) {
                    anyhow::bail!("Failed sening PONG reply")
                }

                Ok(())
            },

            Some((GossipRequest::PublishMe, reply_channel)) => {
                let profile = self.inner.profile();
                let profile = profile.read().await;

                let head = profile.head();
                if head.is_none() {
                    if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::NoHead)) {
                        anyhow::bail!("Failed to send GossipReply::NoHead)")
                    }
                }
                let head = head.unwrap().to_bytes();

                let own_id = match profile.client().own_id().await {
                    Ok(id) => id,
                    Err(e) => if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::PublishMeResult(Err(e)))) {
                        anyhow::bail!("Failed to send GossipReply::PublishMeResult(Err(_))")
                    } else {
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
                    Ok(()) => if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::PublishMeResult(Ok(())))) {
                        anyhow::bail!("Failed to send GossipReply::PublishMeResult(Ok(()))")
                    } else {
                        Ok(())
                    },

                    Err(e) => if let Err(_) = reply_channel.send(ReactorReply::Custom(GossipReply::PublishMeResult(Err(e)))) {
                        anyhow::bail!("Failed to send GossipReply::PublishMeResult(Err(_))")
                    } else {
                        Ok(())
                    }

                }
            },
        }
    }

    pub async fn run(mut self) -> Result<()> {
        use std::convert::TryFrom;
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
                        match serde_json::from_slice(&next_gossip_message.data) {
                            Err(e) => log::trace!("Failed to deserialize gossip message from {}", next_gossip_message.source),
                            Ok(GossipMessage::CurrentProfileState { peer_id, cid }) => {
                                let peer_id = ipfs::PeerId::from_bytes(&peer_id);
                                let cid = cid::Cid::try_from(&*cid);
                                log::trace!("Peer {:?} is at {:?}", peer_id, cid)
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}


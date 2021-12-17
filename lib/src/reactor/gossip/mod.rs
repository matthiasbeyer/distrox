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
use crate::reactor::ReactorBuilder;
use crate::reactor::ctrl::ReactorReceiver;
use crate::reactor::ctrl::ReactorSender;
use crate::reactor::ctrl::ReplySender;

mod ctrl;
pub use ctrl::GossipRequest;
pub use ctrl::GossipReply;

mod msg;
pub use msg::GossipMessage;

#[derive(Debug)]
pub struct GossipReactorBuilder {
    profile: Arc<RwLock<Profile>>,
    gossip_topic_name: String,
}

impl GossipReactorBuilder {
    pub fn new(profile: Arc<RwLock<Profile>>, gossip_topic_name: String) -> Self {
        Self { profile, gossip_topic_name }
    }
}

impl ReactorBuilder for GossipReactorBuilder {
    type Reactor = GossipReactor;

    fn build_with_receiver(self, rr: ReactorReceiver<GossipRequest, GossipReply>) -> Self::Reactor {
        GossipReactor {
            profile: self.profile,
            gossip_topic_name: self.gossip_topic_name,
            receiver: rr,
        }
    }
}

#[derive(Debug)]
pub struct GossipReactor {
    profile: Arc<RwLock<Profile>>,
    gossip_topic_name: String,
    receiver: ReactorReceiver<GossipRequest, GossipReply>,
}

impl GossipReactor {
    fn send_gossip_reply(channel: ReplySender<GossipReply>, reply: GossipReply) -> Result<()> {
        if let Err(_) = channel.send(reply) {
            anyhow::bail!("Failed to send GossipReply::NoHead)")
        }

        Ok(())
    }

    async fn publish_me(&self, reply_channel: ReplySender<GossipReply>) -> Result<()> {
        let profile = self.profile.read().await;

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

    async fn connect(&self, addr: ipfs::MultiaddrWithPeerId) -> Result<()> {
        log::trace!("Connecting GossipReactor with {:?}", addr);
        self.profile.read().await.client().connect(addr).await
    }

    #[cfg(test)]
    async fn is_connected_to(&self, addr: ipfs::MultiaddrWithPeerId) -> Result<bool> {
        self.profile
            .read()
            .await
            .client()
            .ipfs
            .peers()
            .await
            .map(|connections| {
                connections.iter().any(|connection| connection.addr == addr)
            })
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

}

#[async_trait::async_trait]
impl Reactor for GossipReactor {
    type Request = GossipRequest;
    type Reply = GossipReply;

    async fn run(mut self) -> Result<()> {
        use futures::stream::StreamExt;

        log::trace!("Booting {:?}", self);
        let mut subscription_stream = self.profile
            .read()
            .await
            .client()
            .ipfs
            .pubsub_subscribe(self.gossip_topic_name.clone())
            .await?;

        log::trace!("{:?} main loop", self);
        loop {
            tokio::select! {
                next_control_msg = self.receiver.recv() => {
                    log::trace!("Received control message");
                    match next_control_msg {
                        None => break,
                        Some((GossipRequest::Exit, reply_channel)) => {
                            if let Err(_) = reply_channel.send(GossipReply::Exiting) {
                                anyhow::bail!("Failed sending EXITING reply")
                            }
                            break
                        },

                        Some((GossipRequest::Ping, reply_channel)) => {
                            if let Err(_) = reply_channel.send(GossipReply::Pong) {
                                anyhow::bail!("Failed sending PONG reply")
                            }
                        },

                        Some((GossipRequest::PublishMe, reply_channel)) => self.publish_me(reply_channel).await?,

                        Some((GossipRequest::Connect(addr), reply_channel)) => {
                            let reply = GossipReply::ConnectResult(self.connect(addr.clone()).await);
                            if let Err(_) = Self::send_gossip_reply(reply_channel, reply) {
                                anyhow::bail!("Failed sending Connect({}) reply", addr)
                            }
                        },
                    }
                }

                next_gossip_message = subscription_stream.next() => {
                    if let Some(next_gossip_message) = next_gossip_message {
                        log::trace!("Received gossip message");
                        self.handle_gossip_message(next_gossip_message).await?;
                    } else {
                        log::trace!("Gossip stream closed, breaking reactor loop");
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::TryFrom;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use crate::config::Config;

    #[tokio::test]
    async fn test_gossip_reactor_simple() {
        let _ = env_logger::try_init();

        let profile = Profile::new_inmemory(Config::default(), "test-gossip-reactor-simple").await;
        assert!(profile.is_ok());
        let profile = Arc::new(RwLock::new(profile.unwrap()));

        let gossip_topic_name = String::from("test-gossip-reactor-simple-topic");
        let (rx, tx) = tokio::sync::mpsc::unbounded_channel();
        let reactor = GossipReactorBuilder::new(profile.clone(), gossip_topic_name).build_with_receiver(tx);

        let (reply_sender, mut reply_receiver) = tokio::sync::mpsc::unbounded_channel();
        rx.send((GossipRequest::Ping, reply_sender)).unwrap();

        let mut pong_received = false;
        tokio::select! {
            reply = reply_receiver.recv() => {
                match reply {
                    Some(GossipReply::Pong) => {
                        pong_received = true;
                        let (reply_sender, mut reply_receiver) = tokio::sync::mpsc::unbounded_channel();
                        rx.send((GossipRequest::Exit, reply_sender)).unwrap();
                    }
                    Some(r) => {
                        assert!(false, "Expected ReactorReply::Pong, got: {:?}", r);
                    }
                    None => {
                        // nothing
                    }
                }
            },

            reactor_res = reactor.run() => {
                match reactor_res {
                    Ok(()) => assert!(false, "Reactor finished before pong was received"),

                    Err(e) => {
                        assert!(false, "Reactor errored: {:?}", e);
                    }
                }
            }
        }

        assert!(pong_received, "No pong received");
    }

    #[tokio::test]
    async fn test_gossip_reactor_gossipping() {
        let _ = env_logger::try_init();

        let gossip_topic_name = String::from("test-gossip-reactor-gossipping-topic");
        let (left_profile, left_reactor, left_rx) = {
            let profile = Profile::new_inmemory(Config::default(), "test-gossip-reactor-simple-left").await;
            assert!(profile.is_ok());
            let profile = Arc::new(RwLock::new(profile.unwrap()));

            let (rx, tx) = tokio::sync::mpsc::unbounded_channel();
            let reactor = GossipReactorBuilder::new(profile.clone(), gossip_topic_name.clone()).build_with_receiver(tx);
            (profile, reactor, rx)
        };
        log::trace!("Built left GossipReactor");

        let (right_profile, right_reactor, right_rx) = {
            let profile = Profile::new_inmemory(Config::default(), "test-gossip-reactor-simple-right").await;
            assert!(profile.is_ok());
            let profile = Arc::new(RwLock::new(profile.unwrap()));

            let (rx, tx) = tokio::sync::mpsc::unbounded_channel();
            let reactor = GossipReactorBuilder::new(profile.clone(), gossip_topic_name.clone()).build_with_receiver(tx);
            (profile, reactor, rx)
        };
        log::trace!("Built right GossipReactor");

        async fn get_peer_id(profile: Arc<RwLock<Profile>>) -> Result<ipfs::MultiaddrWithPeerId> {
            profile.read()
                .await
                .client()
                .ipfs
                .identity()
                .await
                .map(|(pubkey, addr)| (pubkey.into_peer_id(), addr))
                .and_then(|(peerid, mut addr)| {
                    ipfs::MultiaddrWithPeerId::try_from({
                        addr.pop().expect("At least one address for client")
                    })
                    .map_err(anyhow::Error::from)
                })
        }

        let left_running_reactor = tokio::spawn(async move {
            left_reactor.run().await
        });

        let right_running_reactor = tokio::spawn(async move {
            right_reactor.run().await
        });

        let left_peer_id = get_peer_id(left_profile.clone()).await;
        log::trace!("Left GossipReactor = {:?}", left_peer_id);
        assert!(left_peer_id.is_ok(), "Not ok: {:?}", left_peer_id);
        let left_peer_id = left_peer_id.unwrap();

        let right_peer_id = get_peer_id(right_profile.clone()).await;
        log::trace!("Right GossipReactor = {:?}", right_peer_id);
        assert!(right_peer_id.is_ok(), "Not ok: {:?}", right_peer_id);
        let right_peer_id = right_peer_id.unwrap();

        let (right_reply_sender, mut right_reply_receiver) = tokio::sync::mpsc::unbounded_channel();

        log::trace!("Right GossipReactor should now connect to left GossipReactor");
        right_rx.send((GossipRequest::Connect(left_peer_id), right_reply_sender)).unwrap();

        log::trace!("Right GossipReactor should now connect to left GossipReactor... waiting for reply");
        match tokio::time::timeout(std::time::Duration::from_secs(5), right_reply_receiver.recv()).await {
            Err(_) => assert!(false, "Timeout elapsed when waiting for connection status"),
            Ok(Some(GossipReply::ConnectResult(Ok(())))) => {
                log::trace!("Right GossipReactor is connected");
                assert!(true)
            },
            Ok(Some(other)) => assert!(false, "Expected ConnectResult(Ok(())), recv: {:?}", other),
            Ok(None) => assert!(false, "No reply from right reactor received"),
        }
    }
}

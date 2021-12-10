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
use crate::reactor::ctrl::ReactorReceiver;
use crate::reactor::ctrl::ReactorReply;
use crate::reactor::ctrl::ReactorRequest;
use crate::reactor::ctrl::ReactorSender;
use crate::reactor::ctrl::ReplyChannel;


#[derive(Debug)]
pub struct GossipReactor {
    inner: Reactor<GossipRequest, GossipReply>,
}

#[derive(Debug)]
pub enum GossipRequest {
    Ping,
}

#[derive(Debug)]
pub enum GossipReply {
    Pong,
}

impl GossipReactor {
    pub fn new(profile: Arc<RwLock<Profile>>) -> (Self, ReactorSender<GossipRequest, GossipReply>) {
        let (inner, sender) = Reactor::<GossipRequest, GossipReply>::new(profile);
        (Self { inner }, sender)
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
            }
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            match self.receive_next_message().await {
                None => break,
                Some(tpl) => self.process_reactor_message(tpl).await?,
            }
        }
        Ok(())
    }
}


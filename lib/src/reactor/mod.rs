use std::sync::Arc;
use std::fmt::Debug;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;

mod gossip;
mod device;
mod account;
mod ctrl;

pub use ctrl::ReactorReceiver;
pub use ctrl::ReactorSender;
pub use ctrl::ReplyChannel;

/// Send control messages to the reactor
#[derive(Debug)]
pub enum ReactorRequest<CustomRequest: Debug + Send + Sync> {
    /// check if the reactor still responds
    Ping,

    /// Quit the reactor
    Exit,

    Connect(ipfs::MultiaddrWithPeerId),

    Custom(CustomRequest),
}

#[derive(Debug)]
pub enum ReactorReply<CustomReply: Debug + Send + Sync> {
    Pong,
    Exiting,

    ConnectResult((Result<()>, ipfs::MultiaddrWithPeerId)),

    Custom(CustomReply),
}

/// Reactor type, for running the application logic
///
/// The Reactor runs the whole application logic, that is syncing with other devices, fetching and
/// keeping profile updates of other accounts, communication on the gossipsub topics... etc
#[derive(Debug, getset::Getters, getset::Setters)]
pub(super) struct Reactor<CustomReactorRequest, CustomReactorReply>
    where CustomReactorRequest: Debug + Send + Sync,
          CustomReactorReply: Debug + Send + Sync
{
    #[getset(get = "pub", set = "pub")]
    running: bool,
    profile: Arc<RwLock<Profile>>,
    rx: ReactorReceiver<CustomReactorRequest, CustomReactorReply>,
}

impl<CustomReactorRequest, CustomReactorReply> Reactor<CustomReactorRequest, CustomReactorReply>
    where CustomReactorRequest: Debug + Send + Sync,
          CustomReactorReply: Debug + Send + Sync
{
    pub(super) fn new(profile: Arc<RwLock<Profile>>) -> (Self, ReactorSender<CustomReactorRequest, CustomReactorReply>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let reactor = Reactor {
            running: true,
            profile,
            rx,
        };

        (reactor, tx)
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

    pub(super) async fn receive_next_message(&mut self) -> Option<(ReactorRequest<CustomReactorRequest>, ReplyChannel<CustomReactorReply>)> {
        self.rx.recv().await
    }

    /// Process the request if it is not a specialized request,
    /// return the specialized request if it is one and cannot be processed by this reactor
    /// implementation
    pub(super) async fn process_reactor_message(&mut self, request: (ReactorRequest<CustomReactorRequest>, ReplyChannel<CustomReactorReply>)) -> Result<Option<(CustomReactorRequest, ReplyChannel<CustomReactorReply>)>> {
        match request {
            (ReactorRequest::Ping, reply_channel) => {
                if let Err(_) = reply_channel.send(ReactorReply::Pong) {
                    anyhow::bail!("Failed sending PONG reply")
                }
                Ok(None)
            },

            (ReactorRequest::Exit, reply_channel) => {
                self.running = false;
                if let Err(_) = reply_channel.send(ReactorReply::Exiting) {
                    anyhow::bail!("Failed sending EXITING reply")
                }
                Ok(None)
            },

            (ReactorRequest::Connect(addr), reply_channel) => {
                match self.profile.read().await.client().connect(addr.clone()).await {
                    Ok(()) => if let Err(_) = reply_channel.send(ReactorReply::ConnectResult((Ok(()), addr.clone()))) {
                        anyhow::bail!("Failed sending ConnectResult({}, Ok(()))", addr)
                    } else {
                        Ok(None)
                    }

                    Err(e) => if let Err(_) = reply_channel.send(ReactorReply::ConnectResult((Err(e), addr.clone()))) {
                        anyhow::bail!("Failed sending ConnectResult({}, Err(_))", addr)
                    } else {
                        Ok(None)
                    }
                }
            }

            (ReactorRequest::Custom(c), reply_channel) => {
                Ok(Some((c, reply_channel)))
            }
        }
    }

}

use std::sync::Arc;

use futures::Stream;
use futures::StreamExt;

use crate::gossip::GossipMessage;

pub struct GossipDeserializer<ErrStrategy = LogStrategy>
where
    ErrStrategy: GossipDeserializerErrorStrategy,
{
    strategy: std::marker::PhantomData<ErrStrategy>,
}

impl<ErrStrategy> Default for GossipDeserializer<ErrStrategy>
where
    ErrStrategy: GossipDeserializerErrorStrategy,
{
    fn default() -> Self {
        Self {
            strategy: std::marker::PhantomData,
        }
    }
}

impl<ErrStrategy> GossipDeserializer<ErrStrategy>
where
    ErrStrategy: GossipDeserializerErrorStrategy,
{
    pub fn run<S>(self, input: S) -> impl Stream<Item = (ipfs::PeerId, GossipMessage)>
    where
        S: Stream<Item = Arc<ipfs::PubsubMessage>>,
    {
        input.filter_map(|message| async move {
            log::trace!("Received gossip message");

            match serde_json::from_slice(&message.data).map_err(anyhow::Error::from) {
                Ok(m) => Some((message.source, m)),
                Err(e) => {
                    ErrStrategy::handle_error(e);
                    None
                }
            }
        })
    }
}

pub trait GossipDeserializerErrorStrategy {
    fn handle_error(err: anyhow::Error);
}

pub struct LogStrategy;
impl GossipDeserializerErrorStrategy for LogStrategy {
    fn handle_error(err: anyhow::Error) {
        log::trace!("Error: {}", err);
    }
}

pub struct IgnoreStrategy;
impl GossipDeserializerErrorStrategy for IgnoreStrategy {
    fn handle_error(_: anyhow::Error) {
        // nothing
    }
}

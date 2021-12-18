use anyhow::Result;
use futures::Stream;
use futures::StreamExt;

use crate::gossip::GossipMessage;

pub struct GossipDeserializer<ErrStrategy = LogStrategy>
    where ErrStrategy: GossipDeserializerErrorStrategy
{
    strategy: std::marker::PhantomData<ErrStrategy>,
}

impl<ErrStrategy> GossipDeserializer<ErrStrategy>
    where ErrStrategy: GossipDeserializerErrorStrategy
{
    pub fn new() -> Self {
        Self {
            strategy: std::marker::PhantomData,
        }
    }

    pub fn run<S>(mut self, input: S) -> impl Stream<Item = GossipMessage>
        where S: Stream<Item = ipfs::PubsubMessage>
    {
        input.filter_map(|message| async move {
            log::trace!("Received gossip message");

            match serde_json::from_slice(&message.data).map_err(anyhow::Error::from) {
                Ok(m) => Some(m),
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
        ()
    }
}

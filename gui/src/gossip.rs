use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::RwLock;

use distrox_lib::profile::Profile;
use distrox_lib::client::Client;
use distrox_lib::gossip::GossipDeserializer;
use distrox_lib::gossip::LogStrategy;

use crate::app::Message;

#[derive(Clone, Debug)]
pub struct GossipRecipe {
    profile: Arc<RwLock<Profile>>,
    subscription: Arc<ipfs::SubscriptionStream>,
}

impl GossipRecipe {
    pub fn new(profile: Arc<RwLock<Profile>>, subscription: ipfs::SubscriptionStream) -> Self {
        Self { profile, subscription: Arc::new(subscription) }
    }
}


// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for GossipRecipe
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(self: Box<Self>, _input: futures::stream::BoxStream<'static, I>) -> futures::stream::BoxStream<'static, Self::Output> {
        // TODO: Do "right", whatever this means...
        let stream = Arc::try_unwrap(self.subscription).unwrap();

        Box::pin({
            GossipDeserializer::<LogStrategy>::new()
                .run(stream)
                .map(|(source, msg)| Message::GossipMessage(source, msg))
        })
    }
}

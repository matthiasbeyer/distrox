use std::sync::Arc;

use cid::Cid;

use distrox_lib::gossip::GossipMessage;
use distrox_lib::profile::Profile;
use distrox_lib::types::Payload;

use crate::gossip::GossipRecipe;

#[derive(Clone, Debug)]
pub enum Message {
    Loaded(Arc<Profile>),
    FailedToLoad(String),

    ToggleLog,

    GossipSubscriptionFailed(String),
    GossipHandled(GossipMessage),

    InputChanged(String),
    CreatePost,

    PostCreated(Cid),
    PostCreationFailed(String),

    PostLoaded((Payload, String)),
    PostLoadingFailed,

    TimelineScrolled(f32),
}

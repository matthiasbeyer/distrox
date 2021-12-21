use std::sync::Arc;

use cid::Cid;
use tokio::sync::RwLock;

use distrox_lib::gossip::GossipMessage;
use distrox_lib::profile::Profile;
use distrox_lib::types::Payload;

use crate::gossip::GossipRecipe;

#[derive(Clone, Debug)]
pub enum Message {
    Loaded(Arc<RwLock<Profile>>),
    FailedToLoad(String),
    ProfileStateSaved,
    ProfileStateSavingFailed(String),

    ToggleLog,

    GossipSubscriptionFailed(String),
    GossipHandled(GossipMessage),

    PublishGossipAboutMe,
    OwnStateGossipped,
    GossippingFailed(String),

    InputChanged(String),
    CreatePost,

    PostCreated(Cid),
    PostCreationFailed(String),

    PostLoaded((Payload, String)),
    PostLoadingFailed,

    TimelineScrolled(f32),
}

impl Message {
    pub fn description(&self) -> &'static str {
        match self {
            Message::Loaded(_)                   => "Loaded",
            Message::FailedToLoad(_)             => "FailedToLoad",
            Message::ProfileStateSaved           => "ProfileStateSaved",
            Message::ProfileStateSavingFailed(_) => "ProfileStateSavingFailed",

            Message::ToggleLog                   => "ToggleLog",

            Message::GossipSubscriptionFailed(_) => "GossipSubscriptionFailed",
            Message::GossipHandled(_)            => "GossipHandled",

            Message::PublishGossipAboutMe        => "PublishGossipAboutMe",
            Message::OwnStateGossipped           => "OwnStateGossipped",
            Message::GossippingFailed(_)         => "GossippingFailed",

            Message::InputChanged(_)             => "InputChanged",
            Message::CreatePost                  => "CreatePost",

            Message::PostCreated(_)              => "PostCreated",
            Message::PostCreationFailed(_)       => "PostCreationFailed",

            Message::PostLoaded(_)               => "PostLoaded",
            Message::PostLoadingFailed           => "PostLoadingFailed",

            Message::TimelineScrolled(_)         => "TimelineScrolled",
        }
    }
}


use std::sync::Arc;

use cid::Cid;

use distrox_lib::profile::Profile;
use distrox_lib::types::Payload;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Arc<Profile>),
    FailedToLoad,

    InputChanged(String),
    CreatePost,

    PostCreated(Cid),
    PostCreationFailed(String),

    PostLoaded((Payload, String)),
    PostLoadingFailed,

    TimelineScrolled(f32),
}

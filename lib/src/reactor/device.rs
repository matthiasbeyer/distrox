//! Module for multi-device support functionality,
//! which uses the gossip module for the lower-level handling

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;
use crate::reactor::gossip::GossipReactor;

#[derive(Debug)]
pub struct DeviceReactor(GossipReactor);

impl DeviceReactor {
    pub(super) fn new(profile: Arc<RwLock<Profile>>) -> Self {
        unimplemented!()
    }
}

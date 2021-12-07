//! Module for account handling (following accounts, caching account updates) using the gossip
//! module for the lower-level handling

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;
use crate::reactor::gossip::GossipReactor;

#[derive(Debug)]
pub struct AccountReactor(GossipReactor);

impl AccountReactor {
    pub(super) fn new(profile: Arc<RwLock<Profile>>) -> Self {
        Self(GossipReactor::new(profile))
    }
}

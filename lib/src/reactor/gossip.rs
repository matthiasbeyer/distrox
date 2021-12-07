//! Low-level module for gossip'ing code
//!
//! This module implements the low-level gossiping functionality that other modules use to
//! implement actual behaviours on
//!

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;

#[derive(Debug)]
pub struct GossipReactor {
    profile: Arc<RwLock<Profile>>,
}

impl GossipReactor {
    pub(super) fn new(profile: Arc<RwLock<Profile>>) -> Self {
        Self { profile }
    }
}


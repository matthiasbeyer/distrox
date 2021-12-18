use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;
use crate::reactor::gossip::msg::GossipMessage;

#[async_trait::async_trait]
pub trait GossipHandlingStrategy: Sync + Send {
    async fn handle_gossip_message(profile: Arc<RwLock<Profile>>, source: ipfs::PeerId, msg: GossipMessage) -> Result<()>;
}

pub struct LogStrategy;

#[async_trait::async_trait]
impl GossipHandlingStrategy for LogStrategy {
    async fn handle_gossip_message(profile: Arc<RwLock<Profile>>, source: ipfs::PeerId, msg: GossipMessage) -> Result<()> {
        use std::convert::TryFrom;
        match msg {
            GossipMessage::CurrentProfileState { peer_id, cid } => {
                let peer_id = ipfs::PeerId::from_bytes(&peer_id);
                let cid = cid::Cid::try_from(&*cid);

                log::trace!("{:?} told me that {:?} is at {:?}", source, peer_id, cid);
            }
        }

        Ok(())
    }
}

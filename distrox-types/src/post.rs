use libipld::DagCbor;

use crate::id::ContentId;
use crate::id::NodeId;
use crate::id::PostId;

#[derive(Debug, DagCbor)]
pub enum Post {
    Original(OriginalPost),
    Repost(Repost),
    Announce(Announce),
}

#[derive(Debug, DagCbor)]
pub struct OriginalPost {
    pub content: ContentId,
    pub content_mime: crate::util::Mime,
    pub timestamp: crate::util::OffsetDateTime,
}

#[derive(Debug, DagCbor)]
pub struct Repost {
    pub node_id: NodeId,
    pub post_id: PostId,
}

/// Telling the network about a post from someone else that might be nice to know
#[derive(Debug, DagCbor)]
pub struct Announce {
    pub node_id: NodeId,
    pub post_id: PostId,
}

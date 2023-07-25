use crate::id::ContentId;
use crate::id::NodeId;
use crate::id::PostId;

pub enum Post {
    Original(OriginalPost),
    Repost(Repost),
    Announce(Announce),
}

pub struct OriginalPost {
    pub content: ContentId,
    pub content_mime: mime::Mime,
    pub timestamp: time::OffsetDateTime,
}

pub struct Repost {
    pub node_id: NodeId,
    pub post_id: PostId,
}

/// Telling the network about a post from someone else that might be nice to know
pub struct Announce {
    pub node_id: NodeId,
    pub post_id: PostId,
}

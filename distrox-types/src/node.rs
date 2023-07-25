use crate::id::NodeId;
use crate::id::PostId;
use crate::protocol::ProtocolVersion;

pub struct Node {
    pub protocol_version: ProtocolVersion,

    pub parents: Vec<NodeId>,

    pub post: Option<PostId>,
}

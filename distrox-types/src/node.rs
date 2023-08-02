use libipld::DagCbor;

use crate::id::NodeId;
use crate::id::PostId;
use crate::protocol::ProtocolVersion;

#[derive(Clone, Eq, PartialEq, Debug, DagCbor)]
pub struct Node {
    pub protocol_version: ProtocolVersion,

    pub parents: Vec<NodeId>,

    pub post: Option<PostId>,
}

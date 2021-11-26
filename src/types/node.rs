#[derive(Debug, Eq, PartialEq, libipld::DagCbor)]
pub struct Node {
    /// Version
    v: String,

    /// Parent Nodes, identified by cid
    parents: Vec<crate::types::Id>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: cid::Cid,
}

impl daglib::Node for Node {
    type Id = crate::types::Id;

    fn parent_ids(&self) -> Vec<Self::Id> {
        self.parents.clone()
    }
}

impl Node {
    pub fn version(&self) -> &str {
        &self.v
    }
    pub fn payload_id(&self) -> &cid::Cid {
        &self.payload
    }

    pub fn new(version: String, parents: Vec<crate::types::Id>, payload: cid::Cid) -> Self {
        Node {
            v: version,
            parents,
            payload
        }
    }

}

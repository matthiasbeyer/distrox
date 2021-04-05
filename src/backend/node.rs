#[derive(Debug, Eq, PartialEq, libipld::DagCbor)]
pub struct Node {
    /// Version
    v: String,

    /// Parent Nodes, identified by cid
    parents: Vec<crate::backend::Id>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: cid::Cid,
}

impl daglib::Node for Node {
    type Id = crate::backend::Id;

    fn parent_ids(&self) -> Vec<Self::Id> {
        self.parents.clone()
    }
}


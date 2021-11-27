#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    /// Version
    v: String,

    /// Parent Nodes, identified by cid
    parents: Vec<crate::cid::Cid>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: crate::cid::Cid,
}

impl daglib::Node for Node {
    type Id = crate::cid::Cid;

    fn parent_ids(&self) -> Vec<Self::Id> {
        self.parents.clone()
    }
}

impl Node {
    pub fn new(v: String, parents: Vec<crate::cid::Cid>, payload: crate::cid::Cid) -> Self {
        Self { v, parents, payload }
    }
}

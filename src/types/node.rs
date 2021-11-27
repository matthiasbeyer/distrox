#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, getset::Getters)]
pub struct Node {
    /// Version
    #[serde(rename = "v")]
    #[getset(get = "pub")]
    version: String,

    /// Parent Nodes, identified by cid
    #[getset(get = "pub")]
    parents: Vec<crate::cid::Cid>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: crate::types::encodable_cid::EncodableCid,
}

impl daglib::Node for Node {
    type Id = crate::cid::Cid;

    fn parent_ids(&self) -> Vec<Self::Id> {
        self.parents.clone()
    }
}

impl Node {
    pub fn new(version: String, parents: Vec<crate::cid::Cid>, payload: crate::cid::Cid) -> Self {
        Self { version, parents, payload: payload.into() }
    }

    pub fn payload(&self) -> crate::cid::Cid {
        self.payload.clone().into()
    }
}

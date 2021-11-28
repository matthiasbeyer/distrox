#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, getset::Getters)]
pub struct Node {
    /// Version
    #[serde(rename = "v")]
    #[getset(get = "pub")]
    version: String,

    /// Parent Nodes, identified by cid
    parents: Vec<crate::types::encodable_cid::EncodableCid>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: crate::types::encodable_cid::EncodableCid,
}

impl daglib::Node for Node {
    type Id = crate::cid::Cid;

    fn parent_ids(&self) -> Vec<Self::Id> {
        self.parents()
    }
}

impl Node {
    pub fn new(version: String, parents: Vec<crate::cid::Cid>, payload: crate::cid::Cid) -> Self {
        Self {
            version,
            parents: parents.into_iter().map(crate::types::encodable_cid::EncodableCid::from).collect(),
            payload: payload.into()
        }
    }

    pub fn parents(&self) -> Vec<crate::cid::Cid> {
        self.parents
            .clone()
            .into_iter()
            .map(crate::types::encodable_cid::EncodableCid::into)
            .collect()
    }

    pub fn payload(&self) -> crate::cid::Cid {
        self.payload.clone().into()
    }
}

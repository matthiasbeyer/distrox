#[derive(Clone, Debug, Eq, PartialEq, Hash, libipld::DagCbor)]
pub struct Id(cid::Cid);

impl daglib::NodeId for Id { }

impl From<cid::Cid> for Id {
    fn from(cid: cid::Cid) -> Self {
        Id(cid)
    }
}

impl AsRef<cid::Cid> for Id {
    fn as_ref(&self) -> &cid::Cid {
        &self.0
    }
}

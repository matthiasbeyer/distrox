use libipld::DagCbor;

#[derive(Clone, Debug, Eq, PartialEq, DagCbor)]
pub struct ProtocolVersion(pub u64);

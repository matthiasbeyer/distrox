use libipld::DagCbor;

#[derive(Debug, DagCbor)]
pub struct ProtocolVersion(u64);

use crate::backend::DateTime;
use crate::backend::MimeType;

#[derive(Debug, Eq, PartialEq, libipld::DagCbor)]
pub struct Payload {
    mime: MimeType,
    timestamp: DateTime,
    content: Vec<u8>,
}


use crate::types::DateTime;

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, getset::Getters)]
pub struct Payload {
    // TODO: Make this a mime::Mime, but as this type does not impl Serialize/Deserialize, we
    // cannot do this trivially yet
    #[getset(get = "pub")]
    mime: String,

    #[getset(get = "pub")]
    timestamp: DateTime,

    #[getset(get = "pub")]
    content: crate::cid::Cid,
}

impl Payload {
    pub fn new(mime: String, timestamp: DateTime, content: crate::cid::Cid) -> Self {
        Self { mime, timestamp, content }
    }
}

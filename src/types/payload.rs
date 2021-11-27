use crate::types::DateTime;

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Payload {
    // TODO: Make this a mime::Mime, but as this type does not impl Serialize/Deserialize, we
    // cannot do this trivially yet
    mime: String,

    timestamp: DateTime,
    content: crate::cid::Cid,
}


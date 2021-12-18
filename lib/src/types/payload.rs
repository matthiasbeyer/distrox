use std::convert::TryFrom;

use anyhow::Result;

use crate::types::DateTime;

#[derive(Clone, Debug, Eq, PartialEq, getset::Getters)]
pub struct Payload {
    // TODO: Make this a mime::Mime, but as this type does not impl Serialize/Deserialize, we
    // cannot do this trivially yet
    #[getset(get = "pub")]
    mime: String,

    #[getset(get = "pub")]
    timestamp: DateTime,

    content: ipfs::Cid,
}

impl Into<ipfs::Ipld> for Payload {
    fn into(self) -> ipfs::Ipld {
        let mut map = std::collections::BTreeMap::new();
        map.insert(String::from("mime"), ipfs::Ipld::String(self.mime));
        map.insert(String::from("timestamp"), self.timestamp.into());
        map.insert(String::from("content"), ipfs::Ipld::Link(self.content));
        ipfs::Ipld::Map(map)
    }
}

impl TryFrom<ipfs::Ipld> for Payload {
    type Error = anyhow::Error;

    fn try_from(ipld: ipfs::Ipld) -> Result<Self> {
        let missing_field = |name: &'static str| move || anyhow::anyhow!("Missing field {}", name);
        let field_wrong_type = |name: &str, expty: &str| anyhow::bail!("Field {} has wrong type, expected {}", name, expty);
        match ipld {
            ipfs::Ipld::Map(map) => {
                let mime = match map.get("mime").ok_or_else(missing_field("mime"))? {
                    ipfs::Ipld::String(s) => s.to_owned(),
                    _ => return field_wrong_type("mime", "String")
                };

                let timestamp = map.get("timestamp")
                    .ok_or_else(missing_field("timestamp"))?;
                let timestamp = DateTime::try_from(timestamp.clone())?; // TODO dont clone

                let content = match map.get("content").ok_or_else(missing_field("content"))? {
                    ipfs::Ipld::Link(cid) => cid.clone(),
                    _ => return field_wrong_type("content", "Link")
                };

                Ok(Payload {
                    mime,
                    timestamp,
                    content
                })
            },

            _ => anyhow::bail!("Unexpected type, expected map"),
        }
    }
}

impl Payload {
    pub fn new(mime: String, timestamp: DateTime, content: ipfs::Cid) -> Self {
        Self { mime, timestamp, content: content.into() }
    }

    pub fn content(&self) -> ipfs::Cid {
        self.content.clone()
    }
}

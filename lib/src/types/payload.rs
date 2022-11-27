use crate::error::Error;
use crate::types::DateTime;
use crate::types::FromIPLD;
use crate::types::IntoIPLD;

#[derive(Clone, Debug, Eq, PartialEq, getset::Getters)]
pub struct Payload {
    // TODO: Make this a mime::Mime, but as this type does not impl Serialize/Deserialize, we
    // cannot do this trivially yet
    #[getset(get = "pub")]
    mime: String,

    #[getset(get = "pub")]
    timestamp: DateTime,

    content: libipld::Cid,
}

impl IntoIPLD for Payload {
    fn into_ipld(self) -> libipld::Ipld {
        let mut map = std::collections::BTreeMap::new();
        map.insert(String::from("mime"), libipld::Ipld::String(self.mime));
        map.insert(String::from("timestamp"), self.timestamp.into_ipld());
        map.insert(String::from("content"), libipld::Ipld::Link(self.content));
        libipld::Ipld::Map(map)
    }
}

impl FromIPLD for Payload {
    fn from_ipld(ipld: &libipld::Ipld) -> Result<Self, Error> {
        let missing_field = |name: &'static str| move || Error::MissingField(name.to_string());
        let field_wrong_type =
            |name: &str, expty: &str| Error::WrongFieldType(name.to_string(), expty.to_string());

        match ipld {
            libipld::Ipld::Map(map) => {
                let mime = match map.get("mime").ok_or_else(missing_field("mime"))? {
                    libipld::Ipld::String(s) => s.to_owned(),
                    _ => return Err(field_wrong_type("mime", "String")),
                };

                let timestamp = map
                    .get("timestamp")
                    .ok_or_else(missing_field("timestamp"))?;

                let timestamp = DateTime::from_ipld(&timestamp)?;

                let content = match map.get("content").ok_or_else(missing_field("content"))? {
                    libipld::Ipld::Link(cid) => cid.clone(),
                    _ => return Err(field_wrong_type("content", "Link")),
                };

                Ok(Payload {
                    mime,
                    timestamp,
                    content,
                })
            }

            _ => Err(Error::UnexpectedType("Map".to_string())),
        }
    }
}

impl Payload {
    pub fn new(mime: String, timestamp: DateTime, content: libipld::Cid) -> Self {
        Self {
            mime,
            timestamp,
            content,
        }
    }

    pub fn content(&self) -> libipld::Cid {
        self.content.clone()
    }
}

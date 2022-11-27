use crate::error::Error;
use crate::types::FromIPLD;
use crate::types::IntoIPLD;

#[derive(Debug, Eq, PartialEq, getset::Getters)]
pub struct Node {
    /// Version
    #[getset(get = "pub")]
    version: String,

    /// Parent Nodes, identified by cid
    parents: Vec<libipld::Cid>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: libipld::Cid,
}

impl IntoIPLD for Node {
    fn into_ipld(self) -> libipld::Ipld {
        let mut map = std::collections::BTreeMap::new();
        map.insert(String::from("version"), libipld::Ipld::String(self.version));
        map.insert(
            String::from("parents"),
            libipld::Ipld::List(self.parents.into_iter().map(libipld::Ipld::Link).collect()),
        );
        map.insert(String::from("payload"), libipld::Ipld::Link(self.payload));
        libipld::Ipld::Map(map)
    }
}

impl FromIPLD for Node {
    fn from_ipld(ipld: &libipld::Ipld) -> Result<Self, Error> {
        let missing_field = |name: &'static str| move || Error::MissingField(name.to_string());
        let field_wrong_type =
            |name: &str, expty: &str| Error::WrongFieldType(name.to_string(), expty.to_string());

        match ipld {
            libipld::Ipld::Map(map) => {
                let version = match map.get("version").ok_or_else(missing_field("version"))? {
                    libipld::Ipld::String(s) => s.to_string(),
                    _ => return Err(field_wrong_type("version", "String")),
                };

                let parents = match map.get("parents").ok_or_else(missing_field("parents"))? {
                    libipld::Ipld::List(s) => s
                        .iter()
                        .map(|parent| -> Result<libipld::Cid, Error> {
                            match parent {
                                libipld::Ipld::Link(cid) => Ok(cid.clone()),
                                _ => Err(field_wrong_type("parents", "Link")),
                            }
                        })
                        .collect::<Result<Vec<libipld::Cid>, Error>>()?,
                    _ => return Err(field_wrong_type("parents", "Vec<Link>")),
                };

                let payload = match map.get("payload").ok_or_else(missing_field("payload"))? {
                    libipld::Ipld::Link(cid) => cid.clone(),
                    _ => return Err(field_wrong_type("payload", "Link")),
                };

                Ok(Node {
                    version,
                    parents,
                    payload,
                })
            }

            _ => Err(Error::UnexpectedType("Map".to_string())),
        }
    }
}

impl Node {
    pub fn new(version: String, parents: Vec<libipld::Cid>, payload: libipld::Cid) -> Self {
        Self {
            version,
            parents,
            payload,
        }
    }

    pub fn parents(&self) -> Vec<libipld::Cid> {
        self.parents.clone()
    }

    pub fn payload(&self) -> libipld::Cid {
        self.payload.clone()
    }
}

use anyhow::Result;

use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, getset::Getters)]
pub struct Node {
    /// Version
    #[getset(get = "pub")]
    version: String,

    /// Parent Nodes, identified by cid
    parents: Vec<ipfs::Cid>,

    /// The actual payload of the node, which is stored in another document identified by this cid
    payload: ipfs::Cid,
}

impl From<Node> for ipfs::Ipld {
    fn from(node: Node) -> ipfs::Ipld {
        let mut map = std::collections::BTreeMap::new();
        map.insert(String::from("version"), ipfs::Ipld::String(node.version));
        map.insert(
            String::from("parents"),
            ipfs::Ipld::List(node.parents.into_iter().map(ipfs::Ipld::Link).collect()),
        );
        map.insert(String::from("payload"), ipfs::Ipld::Link(node.payload));
        ipfs::Ipld::Map(map)
    }
}

impl TryFrom<ipfs::Ipld> for Node {
    type Error = anyhow::Error;

    fn try_from(ipld: ipfs::Ipld) -> Result<Self> {
        let missing_field = |name: &'static str| move || anyhow::anyhow!("Missing field {}", name);
        let field_wrong_type = |name: &str, expty: &str| {
            anyhow::bail!("Field {} has wrong type, expected {}", name, expty)
        };
        match ipld {
            ipfs::Ipld::Map(map) => {
                let version = match map.get("version").ok_or_else(missing_field("version"))? {
                    ipfs::Ipld::String(s) => s.to_string(),
                    _ => return field_wrong_type("version", "String"),
                };

                let parents = match map.get("parents").ok_or_else(missing_field("parents"))? {
                    ipfs::Ipld::List(s) => s
                        .into_iter()
                        .map(|parent| -> Result<ipfs::Cid> {
                            match parent {
                                ipfs::Ipld::Link(cid) => Ok(cid.clone()),
                                _ => {
                                    anyhow::bail!("Field in parents has wrong type, expected Link")
                                }
                            }
                        })
                        .collect::<Result<Vec<ipfs::Cid>>>()?,
                    _ => return field_wrong_type("parents", "Vec<Link>"),
                };

                let payload = match map.get("payload").ok_or_else(missing_field("payload"))? {
                    ipfs::Ipld::Link(cid) => cid.clone(),
                    _ => return field_wrong_type("payload", "Link"),
                };

                Ok(Node {
                    version,
                    parents,
                    payload,
                })
            }

            _ => anyhow::bail!("Unexpected type, expected map"),
        }
    }
}

impl Node {
    pub fn new(version: String, parents: Vec<ipfs::Cid>, payload: ipfs::Cid) -> Self {
        Self {
            version,
            parents,
            payload,
        }
    }

    pub fn parents(&self) -> Vec<ipfs::Cid> {
        self.parents.clone()
    }

    pub fn payload(&self) -> ipfs::Cid {
        self.payload.clone()
    }
}

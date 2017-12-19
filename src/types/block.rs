use types::util::IPFSHash;
use types::util::Version;

#[derive(Serialize, Deserialize)]
pub struct Block {
    /// The version of the API in use
    #[serde(rename = "v")]
    version: Version,

    /// The parents of this Profile instance
    ///
    /// Multiple are required for merging.
    #[serde(rename = "parents")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parents: Vec<IPFSHash>,

    /// The content of this block, pointed to by IPFS hash.
    #[serde(rename = "content")]
    content: IPFSHash,
}

impl Block {
    pub fn new(version: Version, parents: Vec<IPFSHash>, content: IPFSHash) -> Self {
        Block { version, parents, content }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn parents(&self) -> &Vec<IPFSHash> {
        &self.parents
    }

    pub fn content(&self) -> &IPFSHash {
        &self.content
    }
}


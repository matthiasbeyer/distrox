use crate::types::util::IPFSHash;
use crate::types::util::Version;
use crate::types::content::Content;
use crate::types::payload::*;
use crate::types::content::LoadedContent;
use crate::repository::repository::Repository;

use failure::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    /// The version of the API in use
    #[serde(rename = "v")]
    version: Version,

    /// The parents of this Profile instance
    ///
    /// Multiple are required for merging.
    #[serde(rename = "parents")]
    #[serde(default)]
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

    pub async fn load(self, r: &Repository) -> Result<LoadedBlock, Error> {
        Ok({
            LoadedBlock {
                version: self.version,
                parents: self.parents,
                content: r.get_content(&self.content)
                    .await?
                    .load(r)
                    .await?
            }
        })
    }
}

impl AsRef<Block> for Block {
    fn as_ref(&self) -> &Self {
        &self
    }
}

#[derive(Debug)]
pub struct LoadedBlock {
    version: Version,
    parents: Vec<IPFSHash>,
    content: LoadedContent,
}

impl LoadedBlock {
    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn parents(&self) -> &Vec<IPFSHash> {
        &self.parents
    }

    pub fn content(&self) -> &LoadedContent {
        &self.content
    }
}


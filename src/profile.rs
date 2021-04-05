use std::collections::HashMap;

use anyhow::Result;
use daglib::Node as _;
use cid::Cid;

use crate::backend::Id;
use crate::backend::Node;
use crate::backend::IpfsEmbedBackend;

pub struct Profile {
    dag: daglib::AsyncDag<Id, Node, IpfsEmbedBackend>,

    cache: HashMap<cid::Cid, LoadedNode>,
}

impl Profile {
    pub async fn load(head: Id) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::load(backend, head).await?;
        let cache = HashMap::new();
        Ok(Profile { dag, cache })
    }

    pub async fn create(node: Node) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::new(backend, node).await?;
        let cache = HashMap::new();
        Ok(Profile { dag, cache })
    }
}


pub struct LoadedNode {
    v: String,
    parents: Vec<crate::backend::Id>,
    payload: crate::backend::Payload,
}

impl LoadedNode {
    async fn load_from_node(backend: &IpfsEmbedBackend, cid: &Cid) -> Result<LoadedNode> {
        let ipfs = backend.ipfs();
        let node = {
            let block = ipfs.fetch(cid).await?;
            block.decode::<libipld::cbor::DagCborCodec, crate::backend::Node>()?
        };

        let payload = {
            let block = ipfs.fetch(node.payload_id()).await?;
            block.decode::<libipld::cbor::DagCborCodec, crate::backend::Payload>()?
        };

        Ok({
            LoadedNode {
                v: node.version().to_string(),
                parents: node.parent_ids().clone(),
                payload
            }
        })
    }
}

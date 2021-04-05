use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use daglib::Node as _;
use daglib::DagBackend;

use crate::backend::Id;
use crate::backend::Node;
use crate::backend::IpfsEmbedBackend;

pub struct Profile {
    dag: daglib::AsyncDag<Id, Node, IpfsEmbedBackend>,

    cache: HashMap<Id, LoadedNode>,
}

impl Profile {
    pub async fn load(head: Id) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::load(backend, head).await?;
        let mut cache = HashMap::new();
        let (id, node) = LoadedNode::load(dag.backend(), dag.head().clone()).await?;
        cache.insert(id, node);
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
    async fn load(backend: &IpfsEmbedBackend, id: Id) -> Result<(Id, LoadedNode)> {
        let (id, node) = backend
            .get(id)
            .await?
            .ok_or_else(|| anyhow!("No node found"))?;

        let payload = {
            let ipfs = backend.ipfs();
            let block = ipfs.fetch(node.payload_id()).await?;
            block.decode::<libipld::cbor::DagCborCodec, crate::backend::Payload>()?
        };

        Ok((id, {
            LoadedNode {
                v: node.version().to_string(),
                parents: node.parent_ids().clone(),
                payload
            }
        }))
    }
}

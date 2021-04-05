use std::sync::Arc;

use anyhow::Result;

use crate::backend::Id;
use crate::backend::Node;

#[derive(Clone)]
pub struct IpfsEmbedBackend {
    ipfs: Arc<ipfs_embed::Ipfs<ipfs_embed::DefaultParams>>,
}

impl IpfsEmbedBackend {
    pub fn ipfs(&self) -> Arc<ipfs_embed::Ipfs<ipfs_embed::DefaultParams>> {
        self.ipfs.clone()
    }
}

#[async_trait::async_trait]
impl daglib::DagBackend<Id, Node> for IpfsEmbedBackend {

    async fn get(&self, id: Id) -> Result<Option<(Id, Node)>> {
        let block = self.ipfs.fetch(id.as_ref()).await?;
        let node = block.decode::<libipld::cbor::DagCborCodec, crate::backend::Node>()?;
        Ok(Some((id, node)))
    }

    async fn put(&mut self, node: Node) -> Result<Id> {
        let block = libipld::block::Block::encode(libipld::cbor::DagCborCodec, libipld::multihash::Code::Blake3_256, &node)?;
        let cid = Id::from(block.cid().clone());
        self.ipfs
            .insert(&block)?
            .await
            .map(|_| cid)
    }
}

impl IpfsEmbedBackend {
    pub async fn new_in_memory(cache_size: u64) -> Result<Self> {
        let in_memory = None; // that's how it works...
        let config = ipfs_embed::Config::new(in_memory, cache_size);

        ipfs_embed::Ipfs::new(config).await.map(Arc::new).map(|ipfs| IpfsEmbedBackend { ipfs })
    }
}

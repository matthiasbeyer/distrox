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
        log::trace!("GET {:?}", id);
        let block = self.ipfs.fetch(id.as_ref(), self.ipfs.peers()).await?;
        let node = block.decode::<libipld::cbor::DagCborCodec, crate::backend::Node>()?;
        Ok(Some((id, node)))
    }

    async fn put(&mut self, node: Node) -> Result<Id> {
        log::trace!("PUT {:?}", node);
        let block = libipld::block::Block::encode(libipld::cbor::DagCborCodec, libipld::multihash::Code::Blake3_256, &node)?;
        let cid = Id::from(block.cid().clone());
        self.ipfs
            .insert(&block)
            .map(|_| cid)
    }
}

impl IpfsEmbedBackend {
    pub async fn new_in_memory(cache_size: u64) -> Result<Self> {
        let in_memory = None; // that's how it works...
        let config = ipfs_embed::Config::new(in_memory, cache_size);

        ipfs_embed::Ipfs::new(config).await.map(Arc::new).map(|ipfs| IpfsEmbedBackend { ipfs })
    }

    pub async fn new_with_config(cfg: ipfs_embed::Config) -> Result<Self> {
        ipfs_embed::Ipfs::new(cfg)
            .await
            .map(Arc::new)
            .map(|ipfs| IpfsEmbedBackend { ipfs })
    }

    pub async fn write_payload(&self, payload: &crate::backend::Payload) -> Result<cid::Cid> {
        let block = libipld::block::Block::encode(libipld::cbor::DagCborCodec, libipld::multihash::Code::Blake3_256, &payload)?;
        self.ipfs
            .insert(&block)
            .map(|_| block.cid().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Payload;

    #[tokio::test]
    async fn test_roundtrip_payload() {
        let backend = IpfsEmbedBackend::new_in_memory(1024).await.unwrap();
        let cid = backend.write_payload(&Payload::now_from_text(String::from("test"))).await.unwrap();

        let payload = backend.ipfs().fetch(&cid, backend.ipfs().peers()).await.unwrap();
        let payload = payload.decode::<libipld::cbor::DagCborCodec, crate::backend::Payload>().unwrap();

        assert_eq!(payload.content(), "test".as_bytes())

    }
}

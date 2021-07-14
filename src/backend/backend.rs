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
        log::trace!("Write payload: {:?}", payload);
        let block = libipld::block::Block::encode(libipld::cbor::DagCborCodec, libipld::multihash::Code::Blake3_256, &payload)?;

        log::trace!("Block = {:?}", block);
        let _ = self.ipfs.insert(&block)?;

        log::trace!("Inserted. CID = {}", block.cid());
        Ok(block.cid().clone())
    }

    pub async fn get_payload(&self, cid: &cid::Cid) -> Result<crate::backend::Payload> {
        let block = self.ipfs.fetch(cid, self.ipfs.peers()).await?;
        log::trace!("Block = {:?}", block);

        let payload = block.decode::<libipld::cbor::DagCborCodec, crate::backend::Payload>()?;
        log::trace!("Payload = {:?}", payload);

        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Payload;

    #[tokio::test]
    async fn test_roundtrip_payload() {
        let _ = env_logger::try_init();

        let backend = IpfsEmbedBackend::new_in_memory(1024).await.unwrap();

        let input_payload = Payload::now_from_text(String::from("test"));
        log::debug!("Input = {:?}", input_payload);
        let cid = backend.write_payload(&input_payload).await.unwrap();
        log::debug!("CID = {:?}", cid);
        let output_payload = backend.get_payload(&cid).await.unwrap();

        assert_eq!(input_payload, output_payload);

    }
}

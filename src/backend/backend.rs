use anyhow::Result;

use crate::backend::Id;
use crate::backend::Node;

pub struct IpfsEmbedBackend {
    ipfs: ipfs_embed::Ipfs<ipfs_embed::DefaultParams>,
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


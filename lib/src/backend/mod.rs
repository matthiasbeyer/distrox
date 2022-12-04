use cid::Cid;
use libipld::Ipld;

#[async_trait::async_trait]
pub trait Backend {
    type Error;

    async fn put(&self, dag: Ipld) -> Result<(), Self::Error>;
    async fn get(&self, cid: Cid) -> Result<Ipld, Self::Error>;
}

#[cfg(feature = "backend-ipfs-api")]
mod ipfs_api;

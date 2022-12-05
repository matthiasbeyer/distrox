use cid::Cid;
use libipld::Ipld;

#[async_trait::async_trait]
pub trait Backend {
    type Error;
    type Key: Key;

    async fn generate_key(&self, name: String) -> Result<Self::Key, Self::Error>;

    async fn put(&self, dag: Ipld) -> Result<cid::Cid, Self::Error>;
    async fn get(&self, cid: Cid) -> Result<Ipld, Self::Error>;
}

pub trait Key
where
    Self: Clone + std::fmt::Debug,
{
    fn name(&self) -> &str;
    fn id(&self) -> &str;
}

#[cfg(feature = "backend-ipfs-api")]
mod ipfs_api;

#[cfg(feature = "backend-ipfs-api")]
pub use ipfs_api::Client;

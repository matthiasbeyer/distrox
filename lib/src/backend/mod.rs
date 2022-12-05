use cid::Cid;
use libipld::Ipld;

#[async_trait::async_trait]
pub trait Backend {
    type Error;
    type Key: Key;

    async fn generate_key(&self, name: String) -> Result<Self::Key, Self::Error>;

    async fn put(&self, dag: Ipld) -> Result<cid::Cid, Self::Error>;
    async fn get(&self, cid: Cid) -> Result<Ipld, Self::Error>;

    async fn pin(&self, cid: Cid) -> Result<(), Self::Error>;

    async fn put_binary(&self, data: Vec<u8>) -> Result<cid::Cid, Self::Error>;

    async fn get_binary(
        &self,
        cid: cid::Cid,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<bytes::Bytes, Self::Error>> + Unpin>,
        Self::Error,
    >;
}

pub trait Key
where
    Self: std::fmt::Debug,
{
    fn name(&self) -> &str;
    fn id(&self) -> &str;
}

#[cfg(feature = "backend-ipfs-api")]
mod ipfs_api;

#[cfg(feature = "backend-ipfs-api")]
pub mod implementation {
    pub use super::ipfs_api::Client;
    pub use super::ipfs_api::Error;
    pub use super::ipfs_api::Key;
}

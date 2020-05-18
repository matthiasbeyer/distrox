use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

use ipfs_api::IpfsClient;
use anyhow::Error;
use futures::future::Future;
use futures::stream::Stream;

use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;
use serde::Serialize;
use serde::de::DeserializeOwned;
use chrono::NaiveDateTime;

use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::payload::Payload;
use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;
use crate::repository::client::TypedClientFassade;


/// High-level Client abstraction
///
/// Still a low-level interface though, because we're still operating on the repository directly.
///
/// Should not be used too extensively, but rather through the "Profile" type, which represents the
/// profile of a user.
#[derive(Debug, Clone)]
pub struct Repository(TypedClientFassade);

impl Deref for Repository {
    type Target = TypedClientFassade;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Repository {
    pub fn new(host: &str, port: u16) -> Result<Repository, Error> {
        TypedClientFassade::new(host, port).map(Repository)
    }

    pub async fn get_block<H>(&self, hash: H) -> Result<Block, Error>
        where H: AsRef<IPFSHash>
    {
        self.0.get_typed(hash).await
    }

    pub async fn put_block<B>(&self, b: B) -> Result<IPFSHash, Error>
        where B: AsRef<Block>
    {
        self.0.put_typed(b.as_ref()).await
    }

    pub async fn get_content<H>(&self, hash: H) -> Result<Content, Error>
        where H: AsRef<IPFSHash>
    {
        self.0.get_typed(hash).await
    }

    pub async fn put_content<C>(&self, c: C) -> Result<IPFSHash, Error>
        where C: AsRef<Content>
    {
        self.0.put_typed(c.as_ref()).await
    }

}


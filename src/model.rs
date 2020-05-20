//! The "model" module implements the Database-layer of the application

use std::io::Cursor;
use std::ops::Deref;
use std::result::Result as RResult;
use std::sync::Arc;

use anyhow::Error;
use chrono::NaiveDateTime;
use failure::Fail;
use futures::future::Future;
use futures::future::FutureExt;
use futures::stream::Stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use ipfs_api::IpfsClient;
use ipfs_api::TryFromUri;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;

use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::payload::Payload;
use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;

#[derive(Clone)]
pub struct Model(Arc<IpfsClient>);

impl std::fmt::Debug for Model{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> RResult<(), std::fmt::Error> {
        write!(f, "Model")
    }
}

impl Model {
    pub fn new(host: &str, port: u16) -> Result<Model, Error> {
        debug!("Creating new Model object: {}:{}", host, port);
        IpfsClient::from_str(&format!("{}:{}", host, port))
            .map(Arc::new)
            .map(|c| Model(c))
            .map_err(|e| Error::from(e.compat()))
    }


    //
    //
    // Low-level interface to the ipfs-api
    //
    //

    pub(crate) async fn get_raw_bytes<H: AsRef<IPFSHash>>(&self, hash: H) -> Result<Vec<u8>, Error> {
        debug!("Get: {}", hash.as_ref());
        self.0
            .clone()
            .cat(hash.as_ref())
            .map_ok(|b| b.to_vec())
            .try_concat()
            .map(|r| r.map_err(|e| anyhow!("UNIMPLEMENTED!()")))
            .await
    }

    pub(crate) async fn put_raw_bytes(&self, data: Vec<u8>) -> Result<IPFSHash, Error> {
        debug!("Put: {:?}", data);
        self.0
            .clone()
            .add(Cursor::new(data))
            .await
            .map(|res| IPFSHash::from(res.hash))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }

    pub(crate) async fn publish(&self, key: &str, hash: &str) -> Result<IPNSHash, Error> {
        debug!("Publish: {:?} -> {:?}", key, hash);
        self.0
            .clone()
            .name_publish(hash, false, None, None, Some(key))
            .await
            .map(|res| IPNSHash::from(res.value))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }

    pub(crate) async fn resolve(&self, ipns: IPNSHash) -> Result<IPFSHash, Error> {
        self.0
            .clone()
            .name_resolve(Some(&ipns), true, false)
            .await
            .map(|res| IPFSHash::from(res.path))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }

    //
    //
    // Generic typed interface
    //
    //

    pub(crate) async fn get_typed<H, D>(&self, hash: H) -> Result<D, Error>
        where H: AsRef<IPFSHash>,
              D: DeserializeOwned
    {
        self.get_raw_bytes(hash)
            .await
            .and_then(|data| {
                debug!("Got data, building object: {:?}", data);

                serde_json::from_slice(&data).map_err(|e| Error::from(e.compat()))
            })
    }

    pub(crate) async fn put_typed<S, Ser>(&self, data: &S) -> Result<IPFSHash, Error>
        where S: AsRef<Ser>,
              Ser: Serialize
    {
        let data = serde_json_to_str(data.as_ref())?;
        self.put_raw_bytes(data.into_bytes()).await
    }

    //
    //
    // Typed interface
    //
    //

    pub async fn get_block<H>(&self, hash: H) -> Result<Block, Error>
        where H: AsRef<IPFSHash>
    {
        self.get_typed(hash).await
    }

    pub async fn put_block<B>(&self, b: B) -> Result<IPFSHash, Error>
        where B: AsRef<Block>
    {
        self.put_typed(b.as_ref()).await
    }

    pub async fn get_content<H>(&self, hash: H) -> Result<Content, Error>
        where H: AsRef<IPFSHash>
    {
        self.get_typed(hash).await
    }

    pub async fn put_content<C>(&self, c: C) -> Result<IPFSHash, Error>
        where C: AsRef<Content>
    {
        self.put_typed(c.as_ref()).await
    }

}

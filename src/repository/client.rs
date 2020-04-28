use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

use ipfs_api::IpfsClient;
use failure::Error;
use failure::err_msg;
use futures::future::Future;
use futures::future::FutureExt;
use futures::stream::Stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;

use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;
use serde::Serialize;
use serde::de::DeserializeOwned;
use chrono::NaiveDateTime;

use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::content::Payload;
use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;


/// Internal ClientFassade types
///
/// Abstracts the procedural interface of IpfsClient calls.
#[derive(Clone)]
struct ClientFassade(Arc<IpfsClient>);

impl ClientFassade {
    fn new(host: &str, port: u16) -> Result<ClientFassade, Error> {
        debug!("Creating new ClientFassade object: {}:{}", host, port);
        IpfsClient::new(host, port)
            .map(Arc::new)
            .map(|c| ClientFassade(c))
            .map_err(Into::into)
    }

    async fn get<H: AsRef<IPFSHash>>(&self, hash: H) -> Result<Vec<u8>, Error> {
        debug!("Get: {}", hash.as_ref());
        self.0
            .clone()
            .cat(hash.as_ref())
            .map_ok(|b| b.to_vec())
            .try_concat()
            .map(|r| r.map_err(Error::from))
            .await
    }

    async fn put(&self, data: Vec<u8>) -> Result<IPFSHash, Error> {
        debug!("Put: {:?}", data);
        self.0
            .clone()
            .add(Cursor::new(data))
            .await
            .map(|res| IPFSHash::from(res.hash))
            .map_err(Into::into)
    }

    async fn publish(&self, key: &str, hash: &str) -> Result<IPNSHash, Error> {
        debug!("Publish: {:?} -> {:?}", key, hash);
        self.0
            .clone()
            .name_publish(hash, false, None, None, Some(key))
            .await
            .map(|res| IPNSHash::from(res.value))
            .map_err(Into::into)
    }
}

/// Client wrapper for working with types directly on the client
#[derive(Clone)]
pub struct TypedClientFassade(ClientFassade);

impl TypedClientFassade {
    pub fn new(host: &str, port: u16) -> Result<TypedClientFassade, Error> {
        ClientFassade::new(host, port).map(TypedClientFassade)
    }

    pub async fn get_raw_bytes<H>(&self, hash: H) -> Result<Vec<u8>, Error>
        where H: AsRef<IPFSHash>
    {
        self.0.get(hash).await
    }

    pub async fn get<H, D>(&self, hash: H) -> Result<D, Error>
        where H: AsRef<IPFSHash>,
              D: DeserializeOwned
    {
        self.0
            .clone()
            .get(hash)
            .await
            .and_then(|data| {
                debug!("Got data, building object: {:?}", data);

                serde_json::from_slice(&data).map_err(Error::from)
            })
    }

    pub async fn put<S, Ser>(&self, data: &S) -> Result<IPFSHash, Error>
        where S: AsRef<Ser>,
              Ser: Serialize
    {
        let client = self.0.clone();

        let data = serde_json_to_str(data.as_ref())?;
        client.put(data.into_bytes()).await
    }

    pub async fn publish(&self, key: &str, hash: &str) -> Result<IPNSHash, Error> {
        self.0.publish(key, hash).await
    }

}

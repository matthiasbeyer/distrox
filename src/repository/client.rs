use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;
use std::result::Result as RResult;

use ipfs_api::IpfsClient;
use anyhow::Error;
use futures::future::Future;
use futures::future::FutureExt;
use futures::stream::Stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use failure::Fail;

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


/// Internal ClientFassade types
///
/// Abstracts the procedural interface of IpfsClient calls.
#[derive(Clone)]
pub struct ClientFassade(Arc<IpfsClient>);

impl std::fmt::Debug for ClientFassade {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> RResult<(), std::fmt::Error> {
        write!(f, "ClientFassade")
    }
}

impl ClientFassade {
    fn new(host: &str, port: u16) -> Result<ClientFassade, Error> {
        debug!("Creating new ClientFassade object: {}:{}", host, port);
        IpfsClient::new(host, port)
            .map(Arc::new)
            .map(|c| ClientFassade(c))
            .map_err(|e| Error::from(e.compat()))
    }

    pub async fn get_raw_bytes<H: AsRef<IPFSHash>>(&self, hash: H) -> Result<Vec<u8>, Error> {
        debug!("Get: {}", hash.as_ref());
        self.0
            .clone()
            .cat(hash.as_ref())
            .map_ok(|b| b.to_vec())
            .try_concat()
            .map(|r| r.map_err(|e| anyhow!("UNIMPLEMENTED!()")))
            .await
    }

    pub async fn put_raw_bytes(&self, data: Vec<u8>) -> Result<IPFSHash, Error> {
        debug!("Put: {:?}", data);
        self.0
            .clone()
            .add(Cursor::new(data))
            .await
            .map(|res| IPFSHash::from(res.hash))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }

    pub async fn publish(&self, key: &str, hash: &str) -> Result<IPNSHash, Error> {
        debug!("Publish: {:?} -> {:?}", key, hash);
        self.0
            .clone()
            .name_publish(hash, false, None, None, Some(key))
            .await
            .map(|res| IPNSHash::from(res.value))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }

    pub async fn resolve(&self, ipns: IPNSHash) -> Result<IPFSHash, Error> {
        self.0
            .clone()
            .name_resolve(Some(&ipns), true, false)
            .await
            .map(|res| IPFSHash::from(res.path))
            .map_err(|e| anyhow!("UNIMPLEMENTED!()"))
    }
}

/// Client wrapper for working with types directly on the client
#[derive(Debug, Clone)]
pub struct TypedClientFassade(ClientFassade);

impl Deref for TypedClientFassade {
    type Target = ClientFassade;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TypedClientFassade {
    pub fn new(host: &str, port: u16) -> Result<TypedClientFassade, Error> {
        ClientFassade::new(host, port).map(TypedClientFassade)
    }

    pub async fn get_typed<H, D>(&self, hash: H) -> Result<D, Error>
        where H: AsRef<IPFSHash>,
              D: DeserializeOwned
    {
        self.0
            .clone()
            .get_raw_bytes(hash)
            .await
            .and_then(|data| {
                debug!("Got data, building object: {:?}", data);

                serde_json::from_slice(&data).map_err(|e| Error::from(e.compat()))
            })
    }

    pub async fn put_typed<S, Ser>(&self, data: &S) -> Result<IPFSHash, Error>
        where S: AsRef<Ser>,
              Ser: Serialize
    {
        let client = self.0.clone();

        let data = serde_json_to_str(data.as_ref())?;
        client.put_raw_bytes(data.into_bytes()).await
    }

}

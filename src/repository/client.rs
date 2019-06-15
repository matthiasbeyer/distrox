use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

use ipfs_api::IpfsClient;
use failure::Error;
use failure::err_msg;
use futures::future::Future;
use futures::stream::Stream;

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

    fn get<H: AsRef<IPFSHash>>(&self, hash: H) -> impl Future<Item = Vec<u8>, Error = Error> {
        debug!("Get: {}", hash.as_ref());
        self.0
            .clone()
            .cat(hash.as_ref())
            .concat2()
            .map_err(Error::from)
            .map(|blob| blob.to_vec())
    }

    fn put(&self, data: Vec<u8>) -> impl Future<Item = IPFSHash, Error = Error> {
        debug!("Put: {:?}", data);
        self.0
            .clone()
            .add(Cursor::new(data))
            .map(|res| IPFSHash::from(res.hash))
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

    pub fn get_raw_bytes<H>(&self, hash: H) -> impl Future<Item = Vec<u8>, Error = Error>
        where H: AsRef<IPFSHash>
    {
        self.0.get(hash)
    }

    pub fn get<H, D>(&self, hash: H) -> impl Future<Item = D, Error = Error>
        where H: AsRef<IPFSHash>,
              D: DeserializeOwned
    {
        self.0
            .clone()
            .get(hash)
            .and_then(|data| {
                debug!("Got data, building object: {:?}", data);

                serde_json::from_slice(&data).map_err(Error::from)
            })
    }

    pub fn put<S, Ser>(&self, data: &S) -> impl Future<Item = IPFSHash, Error = Error>
        where S: AsRef<Ser>,
              Ser: Serialize
    {
        let client = self.0.clone();

        ::futures::future::result(serde_json_to_str(data.as_ref()))
            .map_err(Into::into)
            .and_then(move |d| client.put(d.into_bytes()))
    }

}

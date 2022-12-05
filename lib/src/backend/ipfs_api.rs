use std::net::SocketAddr;
use std::str::FromStr;

use futures::TryStreamExt;
use ipfs_api_backend_hyper::IpfsApi;
use ipfs_api_backend_hyper::TryFromUri;
use libipld::prelude::Decode;
use libipld::prelude::Encode;
use tokio::sync::Mutex;

type IpfsClient = ipfs_api_backend_hyper::IpfsClient<hyper::client::connect::HttpConnector>;

pub struct Client {
    client: Mutex<IpfsClient>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    IpfsApi(#[from] ipfs_api_prelude::Error),

    #[error(transparent)]
    IpfsHyper(#[from] ipfs_api_backend_hyper::Error),

    #[error(transparent)]
    Ipld(#[from] libipld::error::Error),

    #[error(transparent)]
    Cid(#[from] cid::Error),
}

impl Client {
    pub fn new(addr: SocketAddr) -> Result<Self, Error> {
        match addr {
            SocketAddr::V4(addr) => {
                let client = Client {
                    client: Mutex::new(IpfsClient::from_ipv4(http::uri::Scheme::HTTP, addr)?),
                };
                Ok(client)
            }
            SocketAddr::V6(addr) => {
                let client = Client {
                    client: Mutex::new(IpfsClient::from_ipv6(http::uri::Scheme::HTTP, addr)?),
                };
                Ok(client)
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Key {
    name: String,
    id: String,
}

impl super::Key for Key {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[async_trait::async_trait]
impl super::Backend for Client {
    type Error = Error;
    type Key = Key;

    async fn generate_key(&self, name: String) -> Result<Self::Key, Self::Error> {
        self.client
            .lock()
            .await
            .key_gen(&name, ipfs_api_backend_hyper::KeyType::Ed25519, 64) // TODO: is 64 ok?
            .await
            .map(|kres| Key {
                name: kres.name,
                id: kres.id,
            })
            .map_err(Error::from)
    }

    async fn put(&self, dag: libipld::Ipld) -> Result<cid::Cid, Self::Error> {
        let mut buf = Vec::new();
        dag.encode(libipld_cbor::DagCborCodec, &mut buf)?;
        let cursor = std::io::Cursor::new(buf);
        let res = {
            let client = self.client.lock().await;
            client
                .dag_put_with_options(cursor, Default::default())
                .await?
        };
        Ok(cid::Cid::from_str(&res.cid.cid_string)?)
    }

    async fn get(&self, cid: cid::Cid) -> Result<libipld::Ipld, Self::Error> {
        self.client
            .lock()
            .await
            .dag_get(&cid.to_string())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(Error::from)
            .map(std::io::Cursor::new)
            .and_then(|mut cursor| {
                libipld::Ipld::decode(libipld_cbor::DagCborCodec, &mut cursor).map_err(Error::from)
            })
    }

    async fn pin(&self, cid: cid::Cid) -> Result<(), Self::Error> {
        self.client
            .lock()
            .await
            .pin_add(&cid.to_string(), true)
            .await
            .map(|_res| ())
            .map_err(Error::from)
    }

    async fn put_binary(&self, data: Vec<u8>) -> Result<cid::Cid, Self::Error>
    {
        self.client
            .lock()
            .await
            .add(std::io::Cursor::new(data))
            .await
            .map_err(Error::from)
            .and_then(|res| cid::Cid::from_str(&res.hash).map_err(Error::from))
    }
}

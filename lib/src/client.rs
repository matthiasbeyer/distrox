use futures::TryStreamExt;

use crate::backend::Backend;
use crate::backend::Key;
use crate::error::Error;
use crate::types::FromIPLD;
use crate::types::IntoIPLD;

pub struct Client {
    backend: Box<
        dyn Backend<
            Error = crate::backend::implementation::Error,
            Key = crate::backend::implementation::Key,
        >,
    >,
}

impl Client {
    fn new(addr: std::net::SocketAddr) -> Result<Self, Error> {
        let backend = crate::backend::implementation::Client::new(addr)?;

        Ok({
            Self {
                backend: Box::new(backend),
            }
        })
    }

    pub async fn create(
        addr: std::net::SocketAddr,
        keyname: String,
    ) -> Result<(Self, Box<dyn Key>), Error> {
        let client = Self::new(addr)?;
        let key = client.generate_key(keyname).await?;

        Ok((client, key))
    }

    pub async fn generate_key(&self, keyname: String) -> Result<Box<dyn Key>, Error> {
        self.backend
            .generate_key(keyname)
            .await
            .map(|k| Box::new(k) as Box<dyn Key>)
            .map_err(Error::from)
    }

    pub async fn put_text(&self, text: String) -> Result<cid::Cid, Error> {
        self.backend
            .put_binary(text.as_bytes().to_vec())
            .await
            .map_err(Error::from)
    }

    pub async fn put_payload(&self, payload: crate::types::Payload) -> Result<cid::Cid, Error> {
        self.backend
            .put(payload.into_ipld())
            .await
            .map_err(Error::from)
    }

    pub async fn put_node(&self, node: crate::types::Node) -> Result<cid::Cid, Error> {
        self.backend
            .put(node.into_ipld())
            .await
            .map_err(Error::from)
    }

    pub async fn get_text(&self, cid: cid::Cid) -> Result<String, Error> {
        self.backend
            .get_binary(cid)
            .await?
            .map_ok(|bytes| bytes.to_vec())
            .try_concat()
            .await
            .map_err(Error::from)
            .and_then(|v| String::from_utf8(v).map_err(Error::from))
    }

    pub async fn get_payload(&self, cid: cid::Cid) -> Result<crate::types::Payload, Error> {
        self.backend
            .get(cid)
            .await
            .map_err(Error::from)
            .and_then(|ipld| crate::types::Payload::from_ipld(&ipld))
    }

    pub async fn get_node(&self, cid: cid::Cid) -> Result<crate::types::Node, Error> {
        self.backend
            .get(cid)
            .await
            .map_err(Error::from)
            .and_then(|ipld| crate::types::Node::from_ipld(&ipld))
    }
}

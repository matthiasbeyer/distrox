use std::io::Cursor;

use anyhow::Result;
use futures::FutureExt;
use futures::TryFutureExt;
use futures::TryStreamExt;
use ipfs_api_backend_hyper::IpfsApi;

use crate::cid::Cid;
use crate::cid::TryToCid;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;
use crate::types::Node;
use crate::types::Payload;
use crate::types::DateTime;

pub struct Client {
    pub(crate) ipfs: IpfsClient,
    config: Config,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client {{ config: {:?} }}", self.config)
    }
}

impl Client {
    pub fn new(ipfs: IpfsClient, config: Config) -> Self {
        Client {
            ipfs,
            config
        }
    }

    pub async fn post_text_blob(&self, text: String) -> Result<Cid> {
        let reader = Cursor::new(text);

        self.ipfs
            .add(reader)
            .await
            .map_err(anyhow::Error::from)
            .and_then(crate::ipfs_client::backend::response::AddResponse::try_to_cid)
    }

    /// Post a text node
    ///
    /// Pass in the parents if there are any.
    ///
    /// # Note
    ///
    /// Does not verify if the `parents` cids point to actual Nodes!
    ///
    /// # Returns
    ///
    /// Returns the Cid of the newly created node, or an error
    pub async fn post_text_node(&self, parents: Vec<Cid>, text: String) -> Result<Cid> {
        self.post_text_node_with_datetime(parents, text, now()).await
    }

    // For testing
    async fn post_text_node_with_datetime(&self, parents: Vec<Cid>, text: String, datetime: DateTime) -> Result<Cid> {
        let text_blob_cid = self.post_text_blob(text).await?;

        let payload = Payload::new(mime::TEXT_PLAIN_UTF_8.as_ref().to_string(), datetime, text_blob_cid);
        let payload_cid = self.post_payload(payload).await?;

        let node = Node::new(crate::consts::protocol_version(), parents, payload_cid);
        self.post_node(node).await
    }

    async fn post_payload(&self, payload: Payload) -> Result<Cid> {
        self.post_serializable(&payload).await
    }

    async fn post_node(&self, node: Node) -> Result<Cid> {
        self.post_serializable(&node).await
    }

    async fn post_serializable<S: serde::Serialize>(&self, s: &S) -> Result<Cid> {
        let payload_s = serde_json::to_string(s)?;
        let payload_c = Cursor::new(payload_s);
        self.ipfs
            .dag_put(payload_c)
            .await
            .map_err(anyhow::Error::from)
            .and_then(crate::ipfs_client::backend::response::DagPutResponse::try_to_cid)
    }

    pub async fn get_node(&self, cid: Cid) -> Result<Node> {
        self.get_deserializeable::<Node>(cid).await
    }

    pub async fn get_payload(&self, cid: Cid) -> Result<Payload> {
        self.get_deserializeable::<Payload>(cid).await
    }

    async fn get_deserializeable<D: serde::de::DeserializeOwned>(&self, cid: Cid) -> Result<D> {
        let bytes = self.ipfs
            .dag_get(cid.as_ref())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        let s = String::from_utf8(bytes)?;
        serde_json::from_str(&s).map_err(anyhow::Error::from)
    }

    pub async fn get_content_text(&self, cid: Cid) -> Result<String> {
        let bytes = self.ipfs
            .cat(cid.as_ref())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        String::from_utf8(bytes).map_err(anyhow::Error::from)
    }
}

fn now() -> DateTime {
    chrono::offset::Utc::now().into()
}

#[cfg(test)]
mod tests {
    use ipfs_api_backend_hyper::TryFromUri;
    use crate::cid::string_to_cid;
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;
    use crate::types::DateTime;

    fn mkdate(y: i32, m: u32, d: u32, hr: u32, min: u32, sec: u32) -> crate::types::DateTime {
        use chrono::TimeZone;

        chrono::prelude::Utc.ymd(y, m, d).and_hms(hr, min, sec).into()
    }

    #[tokio::test]
    async fn test_post_text_blob() {
        let _ = env_logger::try_init();
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let cid = client.post_text_blob(String::from("text")).await;
        assert!(cid.is_ok());
        assert_eq!(cid.unwrap().as_ref(), "QmY2T5EfgLn8qWCt8eus6VX1gJuAp1nmUSdmoehgMxznAf");
    }

    #[tokio::test]
    async fn test_post_text_node() {
        let _ = env_logger::try_init();
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let datetime = mkdate(2021, 11, 27, 12, 30, 0);

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from("text"), datetime).await;
        assert!(cid.is_ok());
        assert_eq!(cid.unwrap().as_ref(), "bafyreifah3uwad7vm6o2zz3dsluscbjznmlgrgqqstk3s3djrdyvwgsulq");
    }

    #[tokio::test]
    async fn test_post_text_node_roundtrip() {
        let _ = env_logger::try_init();
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let datetime = mkdate(2021, 11, 27, 12, 30, 0);

        let text = "text-roundtrip";

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from(text), datetime.clone()).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        assert_eq!(cid.as_ref(), "bafyreiazg25u4bbymcpebwuadr42lwvhpf7diohojeenmsf3rt42v3kbdy");

        let node = client.get_node(cid).await;
        assert!(node.is_ok());
        let node = node.unwrap();

        assert_eq!(*node.version(), crate::consts::protocol_version());
        assert!(node.parents().is_empty());

        let payload = client.get_payload(node.payload().clone()).await;
        assert!(payload.is_ok());
        let payload = payload.unwrap();

        assert_eq!(payload.mime(), mime::TEXT_PLAIN_UTF_8.as_ref());
        assert_eq!(payload.timestamp(), &datetime);

        let content = client.get_content_text(payload.content().clone()).await;
        assert!(content.is_ok());
        let content = content.unwrap();

        assert_eq!(content, text);
    }

    #[tokio::test]
    async fn test_post_text_chain() {
        let _ = env_logger::try_init();
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let chain_elements = vec![
            (mkdate(2021, 11, 27, 12, 30, 0), "text1", "bafyreiddewrcj6nwfzouxhycypbuqwohb6oev62vjqdko5w7obl5qrwhwm"),
            (mkdate(2021, 11, 27, 12, 31, 0), "text2", "bafyreibcu6wgvh62w4gwpnnbhzoafeog4il4wzqg2zo42ogjqrlnr44pgm"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text3", "bafyreica4fz6spaiuk3nd6ybfquj3ysn6nlxuoxcd54xblibpirisjlhkm"),
        ];

        let mut prev: Option<crate::cid::Cid> = None;
        for (datetime, text, expected_cid) in chain_elements {
            let parents = if let Some(previous) = prev.as_ref() {
                vec![previous.clone()]
            } else {
                Vec::new()
            };

            let cid = client.post_text_node_with_datetime(parents, String::from(text), datetime.clone()).await;
            assert!(cid.is_ok());
            let cid = cid.unwrap();
            assert_eq!(cid.as_ref(), expected_cid);
            prev = Some(cid);
        }
    }

    #[tokio::test]
    async fn test_post_text_dag() {
        let _ = env_logger::try_init();
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ipfs, config);

        async fn post_chain(client: &Client, chain_elements: &Vec<(DateTime, &str, &str)>) {
            let mut prev: Option<crate::cid::Cid> = None;
            for (datetime, text, expected_cid) in chain_elements {
                let parents = if let Some(previous) = prev.as_ref() {
                    vec![previous.clone()]
                } else {
                    Vec::new()
                };

                let cid = client.post_text_node_with_datetime(parents, String::from(*text), datetime.clone()).await;
                assert!(cid.is_ok());
                let cid = cid.unwrap();
                assert_eq!(cid.as_ref(), *expected_cid);
                prev = Some(cid);
            }
        }

        // The following posts a DAG like this:
        //
        // * -- * -- * _
        //              \
        // * -- * -- * -- *
        //              /
        //           * -

        let chain_1_elements = vec![
            (mkdate(2021, 11, 27, 12, 30, 0), "text1", "bafyreiddewrcj6nwfzouxhycypbuqwohb6oev62vjqdko5w7obl5qrwhwm"),
            (mkdate(2021, 11, 27, 12, 31, 0), "text2", "bafyreibcu6wgvh62w4gwpnnbhzoafeog4il4wzqg2zo42ogjqrlnr44pgm"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text3", "bafyreica4fz6spaiuk3nd6ybfquj3ysn6nlxuoxcd54xblibpirisjlhkm"),
        ];

        let chain_2_elements = vec![
            (mkdate(2021, 11, 27, 12, 32, 0), "text4", "bafyreiabwst3adcfyqpknatxcs2buut2qnh3vgawio6a5mnth7u5hf4hgy"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text5", "bafyreih6wmjdpoegs4ibz3gsoec6g56nc63bifo4e4hqub6sjsbetuikhm"),
        ];

        post_chain(&client, &chain_1_elements).await;
        post_chain(&client, &chain_2_elements).await;

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from("text6"), mkdate(2021, 11, 27, 12, 32, 0)).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        assert_eq!(cid.as_ref(), "bafyreihrqhbsmqfkzmbsxvkvwtp4eekeri6m6afejf2wmk6gj64b2qwgsa");

        let parents = vec![
            // latest node in chain_1_elements
            string_to_cid(String::from("bafyreica4fz6spaiuk3nd6ybfquj3ysn6nlxuoxcd54xblibpirisjlhkm")).unwrap(),

            // latest node in chain_2_elements
            string_to_cid(String::from("bafyreica4fz6spaiuk3nd6ybfquj3ysn6nlxuoxcd54xblibpirisjl2km")).unwrap(),

            // single node "text6"
            cid
        ];

        let cid = client.post_text_node_with_datetime(parents, String::from("text7"), mkdate(2021, 11, 27, 12, 32, 0)).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        assert_eq!(cid.as_ref(), "bafyreibnwo6phfbi5m6lzjfiaem4xtvjpeq5nnbsicie6whlnkoakgiyua");
    }

}
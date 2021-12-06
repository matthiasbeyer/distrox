use std::convert::TryFrom;

use anyhow::Context;
use anyhow::Result;
use futures::TryStreamExt;
use ipfs::Cid;

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

    pub async fn exit(self) -> Result<()> {
        self.ipfs.exit_daemon().await;
        Ok(())
    }

    pub async fn connect(&self, peer: ipfs::MultiaddrWithPeerId) -> Result<()> {
        self.ipfs.connect(peer).await
    }

    pub async fn post_text_blob(&self, text: String) -> Result<Cid> {
        self.ipfs
            .put_dag(text.into())
            .await
            .map_err(anyhow::Error::from)
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
        self.post(payload).await
    }

    async fn post_node(&self, node: Node) -> Result<Cid> {
        self.post(node).await
    }

    async fn post<S: Into<ipfs::Ipld>>(&self, s: S) -> Result<Cid> {
        self.ipfs.put_dag(s.into()).await.map_err(anyhow::Error::from)
    }

    pub async fn get_node(&self, cid: Cid) -> Result<Node> {
        self.get::<Node>(cid).await
    }

    pub async fn get_payload(&self, cid: Cid) -> Result<Payload> {
        self.get::<Payload>(cid).await
    }

    async fn get<D: TryFrom<ipfs::Ipld, Error = anyhow::Error>>(&self, cid: Cid) -> Result<D> {
        let ipld = self.ipfs
            .get_dag(ipfs::IpfsPath::new(ipfs::path::PathRoot::Ipld(cid)))
            .await?;

        D::try_from(ipld)
    }

    pub async fn get_content_text(&self, cid: Cid) -> Result<String> {
        struct S(String);
        impl TryFrom<ipfs::Ipld> for S {
            type Error = anyhow::Error;
            fn try_from(ipld: ipfs::Ipld) -> Result<Self> {
                match ipld {
                    ipfs::Ipld::String(s) => Ok(S(s)),
                    _ => anyhow::bail!("Not a string"),
                }
            }
        }

        self.get::<S>(cid).await.map(|v| v.0)
    }
}

fn now() -> DateTime {
    chrono::offset::Utc::now().into()
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use cid::Cid;

    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;
    use crate::types::DateTime;

    fn mkdate(y: i32, m: u32, d: u32, hr: u32, min: u32, sec: u32) -> crate::types::DateTime {
        use chrono::TimeZone;

        chrono::prelude::Utc.ymd(y, m, d).and_hms(hr, min, sec).into()
    }

    async fn mk_ipfs() -> IpfsClient {
        let mut opts = ipfs::IpfsOptions::inmemory_with_generated_keys();
        opts.mdns = false;
        let (ipfs, fut): (ipfs::Ipfs<ipfs::TestTypes>, _) = ipfs::UninitializedIpfs::new(opts).start().await.unwrap();
        tokio::task::spawn(fut);
        ipfs
    }

    #[tokio::test]
    async fn test_post_text_blob() {
        let _ = env_logger::try_init();
        let ipfs  = mk_ipfs().await;
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let cid = client.post_text_blob(String::from("text")).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        let expected_cid = Cid::try_from("bafyreienmqqpz622nxgi7xvcx2jf7p3lyagqkwcj5ieil3mhx2zckfl35u").unwrap();
        assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);
    }

    #[tokio::test]
    async fn test_post_text_node() {
        let _ = env_logger::try_init();
        let ipfs  = mk_ipfs().await;
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let datetime = mkdate(2021, 11, 27, 12, 30, 0);

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from("text"), datetime).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        let expected_cid = Cid::try_from("bafyreidem25zq66ktf42l2sjlxmbz5f66bedw3i4ippshhb3h7dxextfty").unwrap();
        assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);
    }

    #[tokio::test]
    async fn test_post_text_node_roundtrip() {
        let _ = env_logger::try_init();
        let ipfs  = mk_ipfs().await;
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let datetime = mkdate(2021, 11, 27, 12, 30, 0);

        let text = "text-roundtrip";

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from(text), datetime.clone()).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        let expected_cid = Cid::try_from("bafyreicwvx755ysg7zfflxhwhl4d6wuuxmmgfexjfvdhgndiugj37bsphq").unwrap();
        assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);

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
        assert!(content.is_ok(), "not ok: {:?}", content);
        let content = content.unwrap();

        assert_eq!(content, text);
    }

    #[tokio::test]
    async fn test_post_text_chain() {
        let _ = env_logger::try_init();
        let ipfs  = mk_ipfs().await;
        let config = Config::default();
        let client = Client::new(ipfs, config);

        let chain_elements = vec![
            (mkdate(2021, 11, 27, 12, 30, 0), "text1", "bafyreidaxkxog3bssyxxjxlsubgg6wauxbobp7gwyucs6gwzyrtsavb7yu"),
            (mkdate(2021, 11, 27, 12, 31, 0), "text2", "bafyreifsgfl6tvcdn42kihjryg7fpjyjgi4v56bud2m2yniqjrrfn3ils4"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text3", "bafyreifnim44y6zfsc7jrf4xs3lbawlc4qqmk4tgmbqnflbggmvvuvul7a"),
        ];

        let mut prev: Option<ipfs::Cid> = None;
        for (datetime, text, expected_cid) in chain_elements {
            let parents = if let Some(previous) = prev.as_ref() {
                vec![previous.clone()]
            } else {
                Vec::new()
            };

            let cid = client.post_text_node_with_datetime(parents, String::from(text), datetime.clone()).await;
            assert!(cid.is_ok());
            let cid = cid.unwrap();
            let expected_cid = Cid::try_from(expected_cid).unwrap();
            assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);
            prev = Some(cid);
        }
    }

    #[tokio::test]
    async fn test_post_text_dag() {
        let _ = env_logger::try_init();
        let ipfs  = mk_ipfs().await;
        let config = Config::default();
        let client = Client::new(ipfs, config);

        async fn post_chain(client: &Client, chain_elements: &Vec<(DateTime, &str, &str)>) {
            let mut prev: Option<ipfs::Cid> = None;
            for (datetime, text, expected_cid) in chain_elements {
                let parents = if let Some(previous) = prev.as_ref() {
                    vec![previous.clone()]
                } else {
                    Vec::new()
                };

                let cid = client.post_text_node_with_datetime(parents, String::from(*text), datetime.clone()).await;
                assert!(cid.is_ok());
                let cid = cid.unwrap();
                let expected_cid = Cid::try_from(*expected_cid).unwrap();
                assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);
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
            (mkdate(2021, 11, 27, 12, 30, 0), "text1", "bafyreidaxkxog3bssyxxjxlsubgg6wauxbobp7gwyucs6gwzyrtsavb7yu"),
            (mkdate(2021, 11, 27, 12, 31, 0), "text2", "bafyreifsgfl6tvcdn42kihjryg7fpjyjgi4v56bud2m2yniqjrrfn3ils4"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text3", "bafyreifnim44y6zfsc7jrf4xs3lbawlc4qqmk4tgmbqnflbggmvvuvul7a"),
        ];

        let chain_2_elements = vec![
            (mkdate(2021, 11, 27, 12, 32, 0), "text4", "bafyreibfkbslobjydkl3tuiqms7dk243fendyqxi5myqkhxquz7arayuwe"),
            (mkdate(2021, 11, 27, 12, 32, 0), "text5", "bafyreicpzj4lfhzsx5pacp2otk7qyyx353lwsvmkp4aplwgvyisg3y4mjm"),
        ];

        post_chain(&client, &chain_1_elements).await;
        post_chain(&client, &chain_2_elements).await;

        let cid = client.post_text_node_with_datetime(Vec::new(), String::from("text6"), mkdate(2021, 11, 27, 12, 32, 0)).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        let expected_cid = Cid::try_from("bafyreifcpqvxzrgmcbdx5omysjfyupsvjxlrfzww5yh75ld7f7ox3vzno4").unwrap();
        assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);

        let parents = vec![
            // latest node in chain_1_elements
            ipfs::Cid::try_from("bafyreifnim44y6zfsc7jrf4xs3lbawlc4qqmk4tgmbqnflbggmvvuvul7a").unwrap(),

            // latest node in chain_2_elements
            ipfs::Cid::try_from("bafyreicpzj4lfhzsx5pacp2otk7qyyx353lwsvmkp4aplwgvyisg3y4mjm").unwrap(),

            // single node "text6"
            cid
        ];

        let cid = client.post_text_node_with_datetime(parents, String::from("text7"), mkdate(2021, 11, 27, 12, 32, 0)).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();
        let expected_cid = Cid::try_from("bafyreieuac7kvefkiu5ls7tqumaef5qiur7l3moa33ay2kaxxpjmfdjbey").unwrap();
        assert_eq!(cid, expected_cid, "{} != {}", cid, expected_cid);
    }

}

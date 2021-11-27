use std::io::Cursor;

use anyhow::Result;
use futures::FutureExt;
use futures::TryFutureExt;
use ipfs_api_backend_hyper::IpfsApi;

use crate::cid::Cid;
use crate::cid::TryToCid;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

pub struct Client {
    ipfs: IpfsClient,
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
}

#[cfg(test)]
mod tests {
    use ipfs_api_backend_hyper::TryFromUri;
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;

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

}

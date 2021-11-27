use anyhow::Result;
use cid::Cid;

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
    fn post_text_blob_impl(&self, text: String) -> Result<Cid> {
        unimplemented!()
    }
}

#[cfg(test)]
impl Client {
    pub fn post_text_blob(&self, text: String) -> Result<Cid> {
        self.post_text_blob_impl(text)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use crate::config::Config;
    use crate::ipfs_client::IpfsClient;

    #[test]
    fn test_post_text_blob() {
        let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
        let config = Config::default();
        let client = Client::new(ifps, config);

        let cid = Client.post_text_blob(String::from("text"));
        assert!(cid.is_ok());
    }

}

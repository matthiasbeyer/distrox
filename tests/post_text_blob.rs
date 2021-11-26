use distrox::client::Client;
use distrox::config::Config;
use distrox::ipfs_client::IpfsClient;

#[test]
fn test_post_text_blob() {
    let ipfs  = IpfsClient::from_str("http://localhost:5001").unwrap();
    let config = Config::default();
    let client = Client::new(ifps, config);

    let cid = Client.post_text_blob(String::from("text"));
    assert!(cid.is_ok());
}


use distrox::ipfs_client::IpfsClient;

#[test]
fn test_connect_str() {
    let _ = IpfsApi::from_str("http://localhost:5001").unwrap();
}

#[test]
fn test_connect_host_and_port() {
    let _ = IpfsApi::from_host_and_port("localhost", 5001).unwrap();
}


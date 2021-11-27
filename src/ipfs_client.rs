pub type IpfsClient = ipfs_api_backend_hyper::IpfsClient;

#[cfg(test)]
mod tests {
    use super::IpfsClient;

    #[test]
    fn test_connect_str() {
        let _ = IpfsClient::from_str("http://localhost:5001").unwrap();
    }

    #[test]
    fn test_connect_host_and_port() {
        let _ = IpfsClient::from_host_and_port("localhost", 5001).unwrap();
    }
}

pub type IpfsClient = ipfs_api_backend_hyper::IpfsClient;

#[cfg(test)]
mod tests {
    use ipfs_api_backend_hyper::TryFromUri;
    use super::IpfsClient;

    #[test]
    fn test_connect_str() {
        let _ = IpfsClient::from_str("http://localhost:5001").unwrap();
    }

    #[test]
    fn test_connect_host_and_port() {
        let _ = IpfsClient::from_host_and_port(http::uri::Scheme::HTTP, "localhost", 5001).unwrap();
    }
}

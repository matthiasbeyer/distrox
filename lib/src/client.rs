use crate::error::Error;
use crate::low_level_client::LowLevelClient;

pub struct Client {
    low_level: LowLevelClient,
}

impl Client {
    pub async fn new(store_grpc_addr: std::net::SocketAddr) -> Result<Self, Error> {
        LowLevelClient::new(store_grpc_addr)
            .await
            .map(|low_level| Self { low_level })
    }
}

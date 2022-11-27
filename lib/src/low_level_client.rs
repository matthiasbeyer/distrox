use crate::error::Error;

pub struct LowLevelClient {
    store: iroh_rpc_client::StoreClient,
}

impl LowLevelClient {
    pub async fn new(store_grpc_addr: std::net::SocketAddr) -> Result<Self, Error> {
        let addr = iroh_rpc_types::store::StoreClientAddr::GrpcHttp2(store_grpc_addr);
        let store = iroh_rpc_client::StoreClient::new(addr).await?;
        Ok(Self {
            store
        })
    }
}


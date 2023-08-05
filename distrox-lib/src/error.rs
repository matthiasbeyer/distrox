#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ipfs(#[from] rust_ipfs::Error),
}

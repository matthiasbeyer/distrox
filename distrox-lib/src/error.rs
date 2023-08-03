#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ipfs(#[from] rust_ipfs::Error),

    #[error("Cannot parse multiaddr")]
    ParseMultiAddr {
        addr: String,
        #[source]
        source: libp2p::multiaddr::Error,
    },

    #[error("Failed to read configuration")]
    ReadingConfig(#[source] std::io::Error),

    #[error("Failed to parse configuration")]
    ParsingConfig(#[source] toml::de::Error),

    #[error("Failed to read state")]
    ReadingState(#[source] std::io::Error),

    #[error("Failed to parse state")]
    ParsingState(#[source] toml::de::Error),
}

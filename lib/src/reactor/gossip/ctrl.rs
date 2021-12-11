use anyhow::Result;

#[derive(Debug)]
pub enum GossipRequest {
    Ping,
    PublishMe,
    Connect(ipfs::MultiaddrWithPeerId),
}

#[derive(Debug)]
pub enum GossipReply {
    Pong,
    NoHead,
    PublishMeResult(Result<()>),
    ConnectResult(Result<()>),
}


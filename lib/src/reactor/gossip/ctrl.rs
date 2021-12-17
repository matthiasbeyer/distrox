use anyhow::Result;

#[derive(Debug)]
pub enum GossipRequest {
    Exit,
    Ping,
    PublishMe,
    Connect(ipfs::MultiaddrWithPeerId),
}

#[derive(Debug)]
pub enum GossipReply {
    Exiting,
    Pong,
    NoHead,
    PublishMeResult(Result<()>),
    ConnectResult(Result<()>),
}


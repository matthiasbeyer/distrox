use anyhow::Result;

#[derive(Debug)]
pub enum GossipRequest {
    Ping,
    PublishMe,
}

#[derive(Debug)]
pub enum GossipReply {
    Pong,
    NoHead,
    PublishMeResult(Result<()>),
}


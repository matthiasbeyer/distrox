use anyhow::Result;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GossipMessage {
    CurrentProfileState {
        peer_id: Vec<u8>,
        cid: Vec<u8>,
    },
}

impl GossipMessage {
    pub(crate) fn into_bytes(self) -> Result<Vec<u8>> {
        serde_json::to_string(&self)
            .map(String::into_bytes)
            .map_err(anyhow::Error::from)
    }
}

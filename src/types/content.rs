use std::collections::BTreeMap;

use anyhow::Error;

use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;
use crate::types::util::MimeType;
use crate::types::util::Timestamp;
use crate::types::payload::Payload;
use crate::types::payload::LoadedPayload;
use crate::repository::repository::Repository;

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {

    //
    //
    // Metadata about the content
    // --------------------------
    //
    // This metadata should be added to each block version. It is a small amount of bytes, but it
    // makes the aggregation much simpler.
    //
    // In v2 of the API, we might change this and put all this meta-information into variants of
    // `Payload`, if we find that aggregation is fast enough.
    //

    /// A list of IPNS hashes which are posting to this chain (so if a client has one profile
    /// node, it can find the latest profile nodes from all devices a user posts from)
    #[serde(rename = "devices")]
    devices: Vec<IPNSHash>,

    /// Timestamp (UNIX timestamp) when this was created. Can be left out.
    #[serde(rename = "timestamp")]
    #[serde(default)]
    timestamp: Option<Timestamp>,

    /// The payload of the content block
    #[serde(rename = "payload")]
    payload: Payload,

}

impl Content {

    pub fn new(devices: Vec<IPNSHash>, timestamp: Option<Timestamp>, payload: Payload) -> Content {
        Content { devices, timestamp, payload }
    }

    pub fn devices(&self) -> &Vec<IPNSHash> {
        &self.devices
    }

    pub fn timestamp(&self) -> Option<&Timestamp> {
        self.timestamp.as_ref()
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub(crate) fn push_device(&mut self, dev: IPNSHash) {
        self.devices.push(dev);
    }

    pub async fn load(self, r: &Repository) -> Result<LoadedContent, Error> {
        Ok({
            LoadedContent {
                devices: self.devices,
                timestamp: self.timestamp,
                payload: self.payload.load(r).await?
            }
        })
    }

}

impl AsRef<Content> for Content {
    fn as_ref(&self) -> &Self {
        &self
    }
}

#[derive(Debug)]
pub struct LoadedContent {
    devices: Vec<IPNSHash>,
    timestamp: Option<Timestamp>,
    payload: LoadedPayload,
}

impl LoadedContent {
    pub fn devices(&self) -> &Vec<IPNSHash> {
        &self.devices
    }

    pub fn timestamp(&self) -> Option<&Timestamp> {
        self.timestamp.as_ref()
    }

    pub fn payload(&self) -> &LoadedPayload {
        &self.payload
    }
}


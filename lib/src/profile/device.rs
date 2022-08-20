use anyhow::Result;
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct Device {
    device_id: libp2p::identity::ed25519::PublicKey,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeviceSaveable {
    device_id: Vec<u8>,
}

impl TryFrom<Device> for DeviceSaveable {
    type Error = anyhow::Error;

    fn try_from(device: Device) -> Result<Self> {
        Ok(DeviceSaveable {
            device_id: device.device_id.encode().to_vec(),
        })
    }
}

impl TryInto<Device> for DeviceSaveable {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Device> {
        libp2p::identity::ed25519::PublicKey::decode(&self.device_id)
            .map(|device_id| Device { device_id })
            .map_err(anyhow::Error::from)
    }
}

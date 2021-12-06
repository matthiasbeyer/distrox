use std::convert::TryFrom;
use anyhow::Error;
use anyhow::Result;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl Into<ipfs::Ipld> for DateTime {
    fn into(self) -> ipfs::Ipld {
        ipfs::Ipld::String(self.0.to_rfc3339())
    }
}

impl TryFrom<ipfs::Ipld> for DateTime {
    type Error = Error;

    fn try_from(ipld: ipfs::Ipld) -> Result<DateTime> {
        match ipld {
            ipfs::Ipld::String(s) => chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(DateTime)
                .map_err(Error::from),
            _ => anyhow::bail!("Expected string for timestamp"),
        }
    }
}


impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        DateTime(dt)
    }
}


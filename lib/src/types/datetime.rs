use crate::error::Error;

use crate::types::{IntoIPLD, FromIPLD};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl IntoIPLD for DateTime  {
    fn into_ipld(self) -> libipld::Ipld {
        libipld::Ipld::String(self.0.to_rfc3339())
    }
}

impl FromIPLD for DateTime {
    fn from_ipld(ipld: &libipld::Ipld) -> Result<DateTime, Error> {
        match ipld {
            libipld::Ipld::String(ref s) => chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(DateTime)
                .map_err(Error::from),
            _ => Err(Error::ExpectedStringForTimestamp),
        }
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        DateTime(dt)
    }
}

impl DateTime {
    pub fn inner(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

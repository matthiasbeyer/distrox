use crate::error::Error;

use crate::types::{FromIPLD, IntoIPLD};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DateTime(time::OffsetDateTime);

impl IntoIPLD for DateTime {
    fn into_ipld(self) -> libipld::Ipld {
        libipld::Ipld::String(
            self.0
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        )
    }
}

impl FromIPLD for DateTime {
    fn from_ipld(ipld: &libipld::Ipld) -> Result<DateTime, Error> {
        match ipld {
            libipld::Ipld::String(ref s) => {
                time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map(DateTime)
                    .map_err(Error::from)
            }
            _ => Err(Error::ExpectedStringForTimestamp),
        }
    }
}

impl From<time::OffsetDateTime> for DateTime {
    fn from(dt: time::OffsetDateTime) -> Self {
        DateTime(dt)
    }
}

impl DateTime {
    pub fn inner(&self) -> &time::OffsetDateTime {
        &self.0
    }
}

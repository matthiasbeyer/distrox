use anyhow::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        DateTime(dt)
    }
}


impl libipld::codec::Encode<libipld_cbor::DagCborCodec> for DateTime {
    fn encode<W: std::io::Write>(&self, c: libipld_cbor::DagCborCodec, w: &mut W) -> libipld::error::Result<()> {
        self.0.to_rfc3339().encode(c, w).map_err(Error::from)
    }
}

impl libipld::codec::Decode<libipld_cbor::DagCborCodec> for DateTime {
    fn decode<R: std::io::Read + std::io::Seek>(c: libipld_cbor::DagCborCodec, r: &mut R) -> libipld::error::Result<Self> {
        String::decode(c, r)
            .map_err(Error::from)
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(Error::from)
            })
            .map(DateTime)
    }
}


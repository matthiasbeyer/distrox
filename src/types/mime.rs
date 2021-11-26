use anyhow::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct MimeType(mime::Mime);

impl From<mime::Mime> for MimeType {
    fn from(mime: mime::Mime) -> Self {
        MimeType(mime)
    }
}

impl<C: libipld::codec::Codec> libipld::codec::Encode<C> for MimeType {
    fn encode<W: std::io::Write>(&self, _c: C, w: &mut W) -> libipld::error::Result<()> {
        w.write_all(self.0.essence_str().as_bytes()).map_err(Error::from)
    }
}

impl libipld::codec::Decode<libipld_cbor::DagCborCodec> for MimeType {
    fn decode<R: std::io::Read + std::io::Seek>(c: libipld_cbor::DagCborCodec, r: &mut R) -> libipld::error::Result<Self> {
        use std::str::FromStr;

        String::decode(c, r)
            .map_err(Error::from)
            .and_then(|s| mime::Mime::from_str(&s).map_err(Error::from))
            .map(MimeType)
    }
}


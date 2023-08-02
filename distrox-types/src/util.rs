pub use self::mime::*;
pub use self::time::*;

mod mime {
    use libipld::cbor::DagCborCodec;

    #[derive(Debug)]
    pub struct Mime(pub mime::Mime);

    impl libipld::codec::Encode<DagCborCodec> for Mime {
        fn encode<W: std::io::Write>(&self, c: DagCborCodec, w: &mut W) -> libipld::Result<()> {
            self.0.as_ref().encode(c, w)
        }
    }

    impl libipld::codec::Decode<DagCborCodec> for Mime {
        fn decode<R: std::io::Read + std::io::Seek>(
            c: DagCborCodec,
            r: &mut R,
        ) -> libipld::Result<Self> {
            let s = String::decode(c, r)?;
            Ok(Self(s.parse()?))
        }
    }
}

mod time {
    use libipld::cbor::DagCborCodec;

    #[derive(Debug)]
    pub struct OffsetDateTime(pub time::OffsetDateTime);

    impl libipld::codec::Encode<DagCborCodec> for OffsetDateTime {
        fn encode<W: std::io::Write>(&self, c: DagCborCodec, w: &mut W) -> libipld::Result<()> {
            let format = time::format_description::parse(
                "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
            )?;

            self.0.format(&format)?.encode(c, w)
        }
    }

    impl libipld::codec::Decode<DagCborCodec> for OffsetDateTime {
        fn decode<R: std::io::Read + std::io::Seek>(
            c: DagCborCodec,
            r: &mut R,
        ) -> libipld::Result<Self> {
            let format = time::format_description::parse(
                "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
            )?;

            let s: String = String::decode(c, r)?;
            let t = time::OffsetDateTime::parse(&s, &format)?;
            Ok(Self(t))
        }
    }
}

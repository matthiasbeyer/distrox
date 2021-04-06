use crate::backend::DateTime;
use crate::backend::MimeType;

#[derive(Debug, Eq, PartialEq, libipld::DagCbor)]
pub struct Payload {
    mime: MimeType,
    timestamp: DateTime,
    content: Vec<u8>,
}

impl Payload {
    pub fn new(mime: MimeType, timestamp: DateTime) -> Self {
        Payload { mime, timestamp, content: Vec::new() }
    }

    pub fn now_from_text(text: String) -> Payload {
        let mime = MimeType::from(mime::TEXT_PLAIN_UTF_8);
        let timestamp = DateTime::from(chrono::offset::Utc::now());

        Self::new(mime, timestamp).with_content(text.as_bytes().to_vec())
    }

    pub fn with_content(mut self, v: Vec<u8>) -> Self {
        self.content = v;
        self
    }

    pub fn with_mimetype(mut self, mime: MimeType) -> Self {
        self.mime = mime;
        self
    }

    pub fn with_timestamp(mut self, ts: DateTime) -> Self {
        self.timestamp = ts;
        self
    }
}


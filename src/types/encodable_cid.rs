use std::collections::HashMap;

/// An DAG-JSON encodable cid
///
/// this is a hack. DAG-JSON expects a linked CID to be of the form
///
///     "/": "<cid hash>"
///
/// (see https://ipld.io/docs/codecs/known/dag-json/)
///
/// so we have a wrapper type here to make the CID encodable
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EncodableCid(HashMap<String, crate::cid::Cid>);

impl From<crate::cid::Cid> for EncodableCid {
    fn from(cid: crate::cid::Cid) -> Self {
        let mut hm = HashMap::new();
        hm.insert(String::from("/"), cid);
        Self(hm)
    }
}

impl Into<crate::cid::Cid> for EncodableCid {
    fn into(self) -> crate::cid::Cid {
        self.0.get("/").unwrap().clone()
    }
}


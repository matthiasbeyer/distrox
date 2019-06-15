use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

use ipfs_api::IpfsClient;
use failure::Error;
use failure::err_msg;
use futures::future::Future;
use futures::stream::Stream;

use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;
use serde::Serialize;
use serde::de::DeserializeOwned;
use chrono::NaiveDateTime;

use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::content::Payload;
use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;


/// High-level Client abstraction
#[derive(Clone)]
pub struct Repository(TypedClientFassade);

impl Repository {
    pub fn new(host: &str, port: u16) -> Result<Repository, Error> {
        TypedClientFassade::new(host, port).map(Repository)
    }

}

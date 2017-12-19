use std::collections::HashMap;

use types::util::IPFSHash;
use types::util::IPNSHash;

/// TODO: Use rustbreak for persistence layer
#[derive(Serialize, Deserialize, Debug)]
pub struct AppState(HashMap<IPFSKeyName, Data>);

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct IPFSKeyName(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    profile_cache: Vec<ProfileData>,
    connect_nodes: Vec<IPFSHash>, // TODO: stronger type?
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileData {
    names: Vec<IPNSHash>,
    follow: bool,
    block: bool,
    knownnames: Vec<String>,
}


use std::collections::HashMap;

use anyhow::Result;

use crate::backend::Id;
use crate::backend::Node;
use crate::backend::IpfsEmbedBackend;

pub struct Profile {
    dag: daglib::AsyncDag<Id, Node, IpfsEmbedBackend>,

    cache: HashMap<cid::Cid, LoadedNode>,
}

impl Profile {
    pub async fn load(head: Id) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::load(backend, head).await?;
        let cache = HashMap::new();
        Ok(Profile { dag, cache })
    }

    pub async fn create(node: Node) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::new(backend, node).await?;
        let cache = HashMap::new();
        Ok(Profile { dag, cache })
    }
}

use anyhow::Result;

use crate::backend::Id;
use crate::backend::Node;
use crate::backend::IpfsEmbedBackend;

pub struct Profile {
    dag: daglib::AsyncDag<Id, Node, IpfsEmbedBackend>,
}

impl Profile {
    pub async fn load(head: Id) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::load(backend, head).await?;
        Ok(Profile { dag })
    }

    pub async fn create(node: Node) -> Result<Self> {
        let backend = IpfsEmbedBackend::new_in_memory(1000).await?;
        let dag = daglib::AsyncDag::new(backend, node).await?;
        Ok(Profile { dag })
    }
}

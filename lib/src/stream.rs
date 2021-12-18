use anyhow::Result;

use crate::client::Client;
use crate::types::Node;

#[derive(Debug)]
pub struct NodeStreamBuilder {
    state: Vec<cid::Cid>
}

impl NodeStreamBuilder {
    pub fn starting_from(node_cid: cid::Cid) -> Self {
        Self {
            state: vec![node_cid]
        }
    }

    pub fn into_stream(self, client: Client) -> impl futures::stream::Stream<Item = Result<Node>> {
        futures::stream::unfold((client, self.state), move |(client, mut state)| {
                async move {
                    if let Some(node_cid) = state.pop() {
                        match client
                            .get_node(node_cid)
                            .await
                            .map(move |node| {
                                node.parents().iter().for_each(|parent| {
                                    state.push(parent.clone())
                                });

                                (node, state)
                            })
                            .map(Some)
                            .transpose()
                        {
                            Some(Ok((item, state))) => Some((Ok(item), (client, state))),
                            Some(Err(e)) => Some((Err(e), (client, vec![]))),
                            None => None,
                        }
                    } else {
                        None
                    }
                }
            })
    }

}

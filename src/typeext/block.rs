use std::ops::Deref;

use failure::Error;
use futures::Future;
use futures::stream;
use futures::stream::Stream;

use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::util::IPFSHash;
use crate::repository::Repository;

/// Wrapper for Block type which holds a reference to the repository and is thus able to provide
/// convenient functionality out of the box
pub struct BlockExt {
    block: Block,
    repo: Repository,
}

impl Into<Block> for BlockExt {
    fn into(self) -> Block {
        self.block
    }
}

impl Deref for BlockExt {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl BlockExt {
    pub fn from_block(block: Block, repo: Repository) -> Self {
        BlockExt { block, repo }
    }

    pub fn parents(&self) -> impl Stream<Item = Result<BlockExt, Error>> {
        stream::unfold((self.repo.clone(), self.block.parents().clone()), move |(repo, mut state)| {
            async {
                if let Some(hash) = state.pop() {
                    match repo.get_block(hash).await {
                        Ok(block) => {
                            Some((Ok(BlockExt::from_block(block, repo.clone())), (repo, state)))
                        },
                        Err(e) => Some((Err(e), (repo, state))),
                    }
                } else {
                    None
                }
            }
        })
    }

    pub async fn content(&self) -> Result<Content, Error> {
        self.repo.get_content(self.block.content().clone()).await
    }
}

use std::ops::Deref;

use failure::Error;
use futures::Future;
use futures::stream;
use futures::stream::Stream;

use crate::types::block::Block;
use crate::types::content::Content;
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

    pub fn parents(&self) -> impl Stream<Item = BlockExt, Error = Error> {
        let parents = self.block.parents().clone();
        let repo    = self.repo.clone();

        stream::unfold(parents, move |mut state| {
            let repo = repo.clone(); // dont understand why this is necessary
            state.pop().map(move |hash| {
                repo.get_block(hash).map(move |block| {
                    (BlockExt::from_block(block, repo.clone()), state)
                })
            })
        })
    }

    pub fn content(&self) -> impl Future<Item = Content, Error = Error> {
        self.repo.get_content(self.block.content().clone())
    }
}

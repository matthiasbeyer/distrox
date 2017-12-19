use std::collcetions::VecDeque;

use failure::Error;

use types::block::Block;
use types::util::IPFSHash;
use repository::Repository;

/// An iterator for iterating over a chain of blocks
///
pub struct BlockIter<'a> {
    repository: &'a Repository,
    queue: VecDeque<Future<Item = IPFSHash, Error = Error>>,
}

impl<'a> BlockIter<'a> {
    pub fn new(repository: &'a Repository, head: IPFSHash) -> BlockIter<'a> {
        let mut queue = VecDeque::new();
        queue.push_back(head);
        BlockIter { repository, queue }
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = Future<Item = Block, Error = Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.pop_front() {
            self.repository
                .resolve_block(&next)
                .then(|block| {
                    self.queue.extend(block.parents().iter().cloned());
                    block
                })
        }

        None
    }

}


use queues::Queue;
use failure::Error;
use futures::Future;

use crate::repository::Repository;
use crate::types::block::Block;

/// An iterator that iterates over `Block`s using a `Repository`
///
pub struct BlockIterator {
    repo: Repository,
    queue: Queue<impl Future<Block, Error>>,
}

impl BlockIterator {
    pub fn new(repo: Repository, initial: IPFSHash) -> Self {
        Repository {
            queue: {
                let q = Queue::default();
                q.add(repo.get_block(initial));
                q
            },
            repo
        }
    }
}

impl Iterator for BlockIterator {
    type Item = Result<Block, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(next_block) = self.queue.remove() {
            match next_block.wait() {
                Some(block) => {
                    block.parents().iter().for_each(|parent| {
                        self.queue.add({
                            self.repo.get_block(parent)
                        });
                    })

                    Some(block)
                },

                Err(e) => return Some(Err(e)),
            }


        } else {
            None
        }
    }
}

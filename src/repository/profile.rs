use failure::Error;
use futures::stream::{self, Stream};
use futures::future;
use futures::Future;

use crate::types::util::IPNSHash;
use crate::types::util::IPFSHash;
use crate::types::block::Block;
use crate::repository::Repository;

#[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ProfileName(String);

impl From<String> for ProfileName {
    fn from(s: String) -> Self {
        ProfileName(s)
    }
}

/// A profile
///
/// A profile can be _any_ profile, not only the profile of the user
pub struct Profile {
    repository: Repository,
    head: IPFSHash,
}

impl Profile {

    /// Create a new Profile.
    ///
    /// One does not want this most of the time, see `load`. Use this only for creating a
    /// completely new profile
    pub fn new(repository: Repository) -> Result<Self, Error> {
        unimplemented!()
    }

    /// Load a profile from the repository
    pub fn load(repository: Repository, key: IPNSHash) -> Result<Self, Error> {
        unimplemented!()
    }

    pub fn blocks(&self) -> impl Stream<Item = Block, Error = Error> {
        let repo = self.repository.clone();
        stream::unfold(vec![self.head.clone()], move |mut state| {
            let repo = repo.clone();
            state.pop()
                .map(move |hash| {
                    repo.get_block(hash).map(|block| {
                        block.parents().iter().for_each(|parent| {
                            state.push(parent.clone())
                        });
                        (block, state)
                    })
                })
        })
    }
}


/// The profile of the user of the application
///
/// Internally this wraps the `Profile` type, but it provides more functionality, for example
/// posting new content.
///
pub struct UserProfile {
    profile: Profile
}

impl UserProfile {

    /// Create a new Profile.
    ///
    /// One does not want this most of the time, see `load`. Use this only for creating a
    /// completely new profile
    pub fn new(repository: Repository) -> Result<Self, Error> {
        Ok(UserProfile {
            profile: Profile::new(repository)?,
        })
    }

    /// Load a profile from the repository
    pub fn load(repository: Repository, key: IPNSHash) -> Result<Self, Error> {
        Ok(UserProfile {
            profile: Profile::load(repository, key)?,
        })
    }

}

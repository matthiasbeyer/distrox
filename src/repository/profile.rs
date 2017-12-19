use std::collections::LinkedList;

//use futures::future::Future;
//use failure::Error;

// use repository::Repository;
use types::block::Block;
// use types::content::Payload;
// use types::content::Content;
// use types::util::IPFSHash;

pub struct Profile {
    chain: LinkedList<Block>,

    // Accumulated

    // not yet, we do not do in-memory caching in the first prototype
}

impl Profile {
    pub fn new(chain: LinkedList<Block>) -> Self {
        Profile { chain }
    }

    //pub fn find_current_profile_information(&self, repo: &Repository)
    //    -> Option<impl Future<Item = Content, Error = Error>>
    //{
    //    self.chain
    //        .iter()
    //        .map(|obj| repo.resolve_content_profile(obj.content()))
    //        .next()
    //}

    //pub fn posts(&self) -> impl Iterator<Item = &Payload> + Sized {
    //    self.chain
    //        .iter()
    //        .map(Block::content)
    //        .map(Content::payload)
    //        .filter(|pl| is_match!(pl, Payload::Post(..)))
    //}

    //pub fn comments_on_post<'a, H: AsRef<IPFSHash>>(&self, post: H, repo: &Repository)
    //    -> impl Iterator<Item = Future<Item = Content, Error = Error> + Sized>
    //{
    //    self.chain
    //        .iter()
    //        .map(|obj| repo.resolve_content_attached_post_comments(obj.content()))
    //        .filter_map(|cmts| {
    //            cmts.map(|c| {
    //                match c.payload() {
    //                    &Payload::AttachedPostComments {
    //                        ref comments_for,
    //                        ref refs
    //                    } => if comments_for == post.as_ref() {
    //                        Some(refs)
    //                    } else {
    //                        None
    //                    },
    //                    _ => None
    //                }
    //            })
    //        })
    //}

}

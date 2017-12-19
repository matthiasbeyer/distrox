//! TODO: Finalize, this is only an idea
//!

// pub mod iter; // TODO: Implement. Complicated stuff though!
pub mod profile;

use std::io::Cursor;
use std::sync::Arc;

use ipfs_api::IpfsClient;
use failure::Error;
use failure::err_msg;
use futures::future::Future;
use futures::stream::Stream;
use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;
use serde::Serialize;

use types::block::Block;
use types::content::Content;
use types::content::Payload;
use types::util::IPFSHash;
// use repository::iter::BlockIter;
// use repository::profile::Profile;

pub struct Repository {
    client: Arc<IpfsClient>,
}

impl Repository {

    pub fn new(host: &str, port: u16) -> Result<Repository, Error> {
        IpfsClient::new(host, port)
            .map(|c| Repository { client: Arc::new(c) })
            .map_err(Into::into)
    }

    // pub fn get_profile<H: AsRef<IPNSHash>>(&self, hash: H, chain_depth: usize)
    //     -> Result<Profile, Error>
    // {
    //     BlockIter::new(self, (*hash.as_ref()).clone())
    //         .take(chain_depth)
    //         .collect::<Result<LinkedList<Block, Error>>>()
    //         .map(Profile::new)
    // }

    pub fn resolve_plain(&self, hash: &IPFSHash)
        -> impl Future<Item = Vec<u8>, Error = Error>
    {
        self.client
            .cat(hash)
            .concat2()
            .map_err(Error::from)
            .map(|blob| blob.into_bytes().to_vec())
    }

    /// Gets a types::Block from a hash or fails
    pub fn resolve_block(&self, hash: &IPFSHash)
        -> impl Future<Item = Block, Error = Error>
    {
        self.client
            .cat(hash)
            .concat2()
            .map_err(Error::from)
            .and_then(|block| {
                String::from_utf8(block.into_bytes().to_vec())
                    .map_err(Error::from)
                    .and_then(|s| serde_json_from_str(&s).map_err(Error::from))
            })
    }

    /// Gets a types::Content from a hash or fails
    pub fn resolve_content(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        self.client
            .cat(hash)
            .concat2()
            .map_err(Error::from)
            .and_then(|content| {
                String::from_utf8(content.into_bytes().to_vec())
                    .map_err(Error::from)
                    .and_then(|s| serde_json_from_str(&s).map_err(Error::from))
            })
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is None
    #[inline]
    pub fn resolve_content_none(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        self.resolve_content(hash)
            .and_then(|content| {
                match content.payload() {
                    &Payload::None  => Ok(content),
                    _               => Err(err_msg("Content is not None")),
                }
            })
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is Post
    #[inline]
    pub fn resolve_content_post(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        self.resolve_content(hash)
            .and_then(|content| {
                match content.payload() {
                    &Payload::Post {..} => Ok(content),
                    _                   => Err(err_msg("Content is not a Post")),
                }
            })
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is AttachedPostComments
    #[inline]
    pub fn resolve_content_attached_post_comments(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        self.resolve_content(hash)
            .and_then(|content| {
                match content.payload() {
                    &Payload::AttachedPostComments {..} => Ok(content),
                    _                                   => Err(err_msg("Content is not AttachedPostComments")),
                }
            })
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is Profile
    #[inline]
    pub fn resolve_content_profile(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        self.resolve_content(hash)
            .and_then(|content| {
                match content.payload() {
                    &Payload::Profile {..} => Ok(content),
                    _                      => Err(err_msg("Content is not a Profile")),
                }
            })
    }


    //
    // PUT
    //

    pub fn put_plain(&self, data: Vec<u8>)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        self.client
            .clone()
            .add(Cursor::new(data))
            .map(|res| IPFSHash::from(res.hash))
            .map_err(Into::into)
    }

    fn put_serialized<'a, S>(&'a self, s: &'a S)
        -> impl Future<Item = IPFSHash, Error = Error>
            where S: Serialize
    {
        let client = self.client.clone();
        let data   = serde_json_to_str(&s);

        ::futures::future::result(data)
            .map_err(Into::into)
            .and_then(move |data| {
                client
                    .add(Cursor::new(data))
                    .map(|res| IPFSHash::from(res.hash))
                    .map_err(Into::into)
            })
    }

    pub fn put_block<'a>(&'a self, block: &'a Block)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        let client = self.client.clone();
        let data   = serde_json_to_str(block);

        ::futures::future::result(data)
            .map_err(Into::into)
            .and_then(move |data| {
                client
                    .add(Cursor::new(data))
                    .map(|res| IPFSHash::from(res.hash))
                    .map_err(Into::into)
            })
    }

    pub fn put_content<'a>(&'a self, content: &'a Content)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        let client = self.client.clone();
        let data   = serde_json_to_str(content);

        ::futures::future::result(data)
            .map_err(Into::into)
            .and_then(move |data| {
                client
                    .add(Cursor::new(data))
                    .map(|res| IPFSHash::from(res.hash))
                    .map_err(Into::into)
            })
    }

}

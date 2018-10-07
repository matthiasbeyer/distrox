//! TODO: Finalize, this is only an idea
//!

// pub mod iter; // TODO: Implement. Complicated stuff though!
pub mod profile;

use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

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
use version::protocol_version;

// use repository::iter::BlockIter;
// use repository::profile::Profile;

pub struct Repository {
    client: Arc<IpfsClient>,
}

#[derive(Debug, Clone)]
pub struct ProfileName(String);

impl Deref for ProfileName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug, Clone)]
pub struct ProfileKey(String);

impl Deref for ProfileKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

    /// The default lifetime for name publishing (profile announcements)
    ///
    /// 10 minutes
    pub fn profile_announce_default_lifetime() -> &'static str {
        "10m"
    }

    /// The default TTL for name publishing (profile announcements)
    ///
    /// 10 minutes
    pub fn profile_announce_default_ttl() -> &'static str {
        "10m"
    }

    /// Announce a profile as current
    ///
    /// Profile identified by IPFS hash.
    ///
    /// Lifetime and TTL are _not_ set to the default in the implementation of this function, but
    /// the IPFS defaults apply (set by the IPFS daemon)
    ///
    pub fn announce_profile<'a>(&'a self,
                                key: ProfileKey,
                                state: &IPFSHash,
                                lifetime: Option<String>,
                                ttl: Option<String>)
        -> impl Future<Item = (), Error = Error>
    {
        let name   = format!("/ipfs/{}", state);
        let client = self.client.clone();

        self.resolve_content_profile(state)
            .and_then(|_| {
                client.name_publish(&name, false, lifetime.as_ref(), ttl.as_ref(), Some(&key))
                    .map_err(From::from)
                    .map(|_| ())
            })
    }

    pub fn new_profile<'a, N>(&'a self,
                              keyname: N,
                              profile: &Content,
                              lifetime: Option<&str>,
                              ttl: Option<&str>)
        -> impl Future<Item = (ProfileName, ProfileKey), Error = Error>
        where N: AsRef<str>
    {
        use ipfs_api::KeyType;

        if !is_match!(profile.payload(), Payload::Profile { .. }) {
            return ::futures::future::err(err_msg(format!("Not a Profile: {:?}", profile)))
        }

        let client = self.client.clone();
        client
            .key_gen(keyname.as_ref(), KeyType::Rsa, 4096)
            .map(|kp| (kp.name, kp.id))
            .and_then(|(key_name, key_id)| { // put the content into IPFS
                self.put_content(profile)
                    .map(|content_hash| (content_hash, key_name, key_id))
            })
            .and_then(|(content_hash, key_name, key_id)| { // put the content into a new block
                let block = Block::new(protocol_version(),
                                       vec![], // no parents for new profile
                                       content_hash);

                self.put_block(&block)
                    .map(|block_hash| (block_hash, key_name, key_id))
            })
            .and_then(|(block_hash, key_name, key_id)| {
                let path = format!("/ipfs/{}", block_hash);
                client
                    .name_publish(&path, false, lifetime, ttl, Some(&key_name))
                    .map(|_publish_response| {
                        (key_name, key_id)
                    })
            })
    }

}

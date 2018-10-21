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
use chrono::NaiveDateTime;

use types::block::Block;
use types::content::Content;
use types::content::Payload;
use types::util::IPFSHash;
use types::util::IPNSHash;
use version::protocol_version;

// use repository::iter::BlockIter;
// use repository::profile::Profile;

mod client;

pub struct Repository {
    client: Arc<IpfsClient>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileName(String);

impl Deref for ProfileName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ProfileName {
    fn from(s: String) -> Self {
        ProfileName(s)
    }
}

pub type ProfileKey = IPNSHash;


impl Repository {

    pub fn new(host: &str, port: u16) -> Result<Repository, Error> {
        debug!("Creating new Repository object: {}:{}", host, port);
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
        debug!("Resolving plain: {}", hash);
        ::repository::client::resolve_plain(self.client.clone(), hash)
    }

    /// Gets a types::Block from a hash or fails
    pub fn resolve_block(&self, hash: &IPFSHash)
        -> impl Future<Item = Block, Error = Error>
    {
        debug!("Resolving block: {}", hash);
        ::repository::client::resolve_block(self.client.clone(), hash)
    }

    pub fn resolve_latest_block(&self, hash: &IPNSHash)
        -> impl Future<Item = Block, Error = Error>
    {
        debug!("Resolving latest block: {}", hash);
        ::repository::client::resolve_latest_block(self.client.clone(), hash)
    }

    /// Gets a types::Content from a hash or fails
    pub fn resolve_content(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        debug!("Resolving content: {}", hash);
        ::repository::client::resolve_content(self.client.clone(), hash)
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is None
    #[inline]
    pub fn resolve_content_none(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        debug!("Resolving content (none): {}", hash);
        ::repository::client::resolve_content_none(self.client.clone(), hash)
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is Post
    #[inline]
    pub fn resolve_content_post(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        debug!("Resolving content (post): {}", hash);
        ::repository::client::resolve_content_post(self.client.clone(), hash)
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is AttachedPostComments
    #[inline]
    pub fn resolve_content_attached_post_comments(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        debug!("Resolving content (attached post comments): {}", hash);
        ::repository::client::resolve_content_attached_post_comments(self.client.clone(), hash)
    }

    /// Helper over Self::resolve_content() which ensures that the Content payload is Profile
    #[inline]
    pub fn resolve_content_profile(&self, hash: &IPFSHash)
        -> impl Future<Item = Content, Error = Error>
    {
        debug!("Resolving content (profile): {}", hash);
        ::repository::client::resolve_content_profile(self.client.clone(), hash)
    }


    //
    // PUT
    //

    pub fn put_plain(&self, data: Vec<u8>)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        debug!("Putting plain");
        ::repository::client::put_plain(self.client.clone(), data)
    }

    fn put_serialized<'a, S>(&'a self, s: &'a S)
        -> impl Future<Item = IPFSHash, Error = Error>
            where S: Serialize
    {
        debug!("Putting serializable object");
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
        debug!("Putting block: {:?}", block);
        ::repository::client::put_block(self.client.clone(), block)
    }

    pub fn put_content<'a>(&'a self, content: &'a Content)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        debug!("Putting content: {:?}", content);
        ::repository::client::put_content(self.client.clone(), content)
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
        debug!("Announcing profile: key: {key:?}, state: {state:?}, lifetime: {lifetime:?}, ttl: {ttl:?}",
               key = key, state = state, lifetime = lifetime, ttl = ttl);
        ::repository::client::announce_profile(self.client.clone(), key, state, lifetime, ttl)
    }

    pub fn new_profile<'a>(&'a self,
                           keyname: String,
                           profile: Content,
                           lifetime: Option<String>,
                           ttl: Option<String>)
        -> impl Future<Item = (ProfileName, ProfileKey), Error = Error>
    {
        use ipfs_api::KeyType;

        debug!("Creating new profile: key: {key:?}, profile: {profile:?}, lifetime: {lifetime:?}, ttl: {ttl:?}",
               key = keyname, profile = profile, lifetime = lifetime, ttl = ttl);

        if !is_match!(profile.payload(), Payload::Profile { .. }) {
            let out = ::futures::future::err(err_msg(format!("Not a Profile: {:?}", profile)));
            return ::futures::future::Either::B(out)
        }

        let client = self.client.clone();
        let result = ::repository::client::new_profile(client, keyname, profile, lifetime, ttl);

        ::futures::future::Either::A(result)
    }

    pub fn new_text_post<'a>(&'a self,
                        publish_key_id: ProfileKey,
                        latest_block: IPFSHash,
                        text: String,
                        time: Option<NaiveDateTime>)
        -> impl Future<Item = (), Error = Error>
    {
        debug!("New text post under {:?}, after block {:?}", publish_key_id, latest_block);
        ::repository::client::new_text_post(self.client.clone(),
                                            publish_key_id,
                                            latest_block,
                                            text,
                                            time)
    }

    pub fn get_key_id_from_key_name<'a>(&'a self, name: ProfileName)
        -> impl Future<Item = ProfileKey, Error = Error>
    {
        ::repository::client::get_key_id_from_key_name(self.client.clone(), name)
    }

    pub fn deref_ipfs_hash<'a>(&'a self, hash: &IPNSHash)
        -> impl Future<Item = IPFSHash, Error = Error>
    {
        ::repository::client::deref_ipfs_hash(self.client.clone(), hash)
    }

}

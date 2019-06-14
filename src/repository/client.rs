use std::io::Cursor;
use std::sync::Arc;
use std::ops::Deref;

use ipfs_api::IpfsClient;
use ipfs_api::KeyType;
use failure::Error;
use failure::err_msg;
use futures::future::Future;
use futures::stream::Stream;
use serde_json::from_str as serde_json_from_str;
use serde_json::to_string as serde_json_to_str;
use chrono::NaiveDateTime;
use itertools::Itertools;

use types::block::Block;
use types::content::Content;
use types::content::Payload;
use types::util::IPFSHash;
use types::util::IPNSHash;
use types::util::Timestamp;
use repository::{ProfileName, ProfileKey};
use version::protocol_version;

pub fn deref_ipns_hash(client: Arc<IpfsClient>,
                     hash: &IPNSHash)
    -> impl Future<Item = IPFSHash, Error = Error>
{
    client
        .name_resolve(Some(hash.deref()), false, false)
        .map_err(Error::from)
        .map(|resp| IPFSHash::from(resp.path))
}

pub fn resolve_plain(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Vec<u8>, Error = Error>
{
    client
        .cat(hash)
        .concat2()
        .map_err(Error::from)
        .map(|blob| blob.to_vec())
}

pub fn resolve_block(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Block, Error = Error>
{
    client
        .cat(hash)
        .concat2()
        .map_err(Error::from)
        .and_then(|block| {
            debug!("Got Block data, building Block object");

            String::from_utf8(block.to_vec())
                .map_err(Error::from)
                .and_then(|s| serde_json_from_str(&s).map_err(Error::from))
        })
}

pub fn get_key_id_from_key_name(client: Arc<IpfsClient>, name: ProfileName)
    -> impl Future<Item = ProfileKey, Error = Error>
{
    client.key_list()
        .map_err(Error::from)
        .and_then(move |list| {
            list.keys
                .into_iter()
                .filter(|pair| pair.name == *name.deref())
                .next()
                .map(|pair| ProfileKey::from(pair.id))
                .ok_or_else(|| err_msg("No Key"))
        })
}

pub fn resolve_latest_block(client: Arc<IpfsClient>, hash: &IPNSHash)
    -> impl Future<Item = Block, Error = Error>
{
    deref_ipns_hash(client.clone(), hash)
        .map_err(Error::from)
        .and_then(|ipfs_hash| resolve_block(client, &ipfs_hash))
}

pub fn resolve_content(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Content, Error = Error>
{
    client
        .cat(hash)
        .concat2()
        .map_err(Error::from)
        .and_then(|content| {
            debug!("Got Content data, building Content object");

            String::from_utf8(content.to_vec())
                .map_err(Error::from)
                .and_then(|s| serde_json_from_str(&s).map_err(Error::from))
        })
}

pub fn resolve_content_none(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Content, Error = Error>
{
    resolve_content(client, hash).and_then(|content| {
        debug!("Got Content object, checking whether it is None");
        match content.payload() {
            &Payload::None  => Ok(content),
            _               => Err(err_msg("Content is not None")),
        }
    })
}

pub fn resolve_content_post(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Content, Error = Error>
{
    resolve_content(client, hash)
        .and_then(|content| {
            debug!("Got Content object, checking whether it is Post");
            match content.payload() {
                &Payload::Post {..} => Ok(content),
                _                   => Err(err_msg("Content is not a Post")),
            }
        })
}

pub fn resolve_content_attached_post_comments(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Content, Error = Error>
{
    resolve_content(client, hash)
        .and_then(|content| {
            debug!("Got Content object, checking whether it is AttachedPostComments");
            match content.payload() {
                &Payload::AttachedPostComments {..} => Ok(content),
                _                                   => Err(err_msg("Content is not AttachedPostComments")),
            }
        })
}

pub fn resolve_content_profile(client: Arc<IpfsClient>, hash: &IPFSHash)
    -> impl Future<Item = Content, Error = Error>
{
    resolve_content(client, hash)
        .and_then(|content| {
            debug!("Got Content object, checking whether it is Profile");
            match content.payload() {
                &Payload::Profile {..} => Ok(content),
                _                      => Err(err_msg("Content is not a Profile")),
            }
        })
}

pub fn announce_block(client: Arc<IpfsClient>,
                      key: ProfileKey,
                      state: &IPFSHash,
                      lifetime: Option<String>,
                      ttl: Option<String>)
    -> impl Future<Item = (), Error = Error>
{
    let name   = format!("/ipfs/{}", state);

    resolve_block(client.clone(), state)
        .and_then(move |_| {
            debug!("Publishing block.");
            client.name_publish(&name,
                                false,
                                lifetime.as_ref().map(String::deref),
                                ttl.as_ref().map(String::deref),
                                Some(&key))
                .map_err(From::from)
                .map(|_| ())
        })
}


pub fn put_plain(client: Arc<IpfsClient>, data: Vec<u8>)
    -> impl Future<Item = IPFSHash, Error = Error>
{
    client
        .add(Cursor::new(data))
        .map(|res| IPFSHash::from(res.hash))
        .map_err(Into::into)
}

pub fn put_block(client: Arc<IpfsClient>, block: &Block)
    -> impl Future<Item = IPFSHash, Error = Error>
{
    let data = serde_json_to_str(block);

    ::futures::future::result(data)
        .map_err(Into::into)
        .and_then(move |data| put_plain(client, data.into_bytes()))
}

pub fn put_content(client: Arc<IpfsClient>, content: &Content)
    -> impl Future<Item = IPFSHash, Error = Error>
{
    let data = serde_json_to_str(content);
    ::futures::future::result(data)
        .map_err(Into::into)
        .and_then(move |data| put_plain(client, data.into_bytes()))
}

pub fn new_profile(client: Arc<IpfsClient>,
                   keyname: String,
                   profile: Content,
                   lifetime: Option<String>,
                   ttl: Option<String>)
    -> impl Future<Item = (ProfileName, ProfileKey), Error = Error>
{
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();

    client
        .key_gen(&keyname, KeyType::Rsa, 4096)
        .map_err(Error::from)
        .map(|kp| (kp.name, kp.id))
        .and_then(move |(key_name, key_id)| { // put the content into IPFS
            let mut prof = profile;
            prof.push_device(IPNSHash::from(key_id.clone()));

            put_content(client1, &prof)
                .map(move |content_hash| (content_hash, key_name, key_id))
                .map_err(Error::from)
        })
        .map(|(content_hash, key_name, key_id)| {
            let block = Block::new(protocol_version(),
                                   vec![], // no parents for new profile
                                   content_hash);

            (block, key_name, key_id)
        })
        .and_then(move |(block, key_name, key_id)| { // put the content into a new block
            put_block(client2, &block)
                .map(|block_hash| (block_hash, key_name, key_id))
                .map_err(Error::from)
        })
        .map(|(block_hash, key_name, key_id)| {
            (format!("/ipfs/{}", block_hash), key_name, key_id)
        })
        .and_then(move |(path, key_name, key_id)| {
            client3
                .name_publish(&path,
                              false,
                              lifetime.as_ref().map(String::deref),
                              ttl.as_ref().map(String::deref),
                              Some(&key_name))
                .map(|_publish_response| {
                    (ProfileName(key_name), ProfileKey::from(key_id))
                })
                .map_err(Error::from)
        })
}

pub fn new_text_post(client: Arc<IpfsClient>,
                     publish_key_id: ProfileKey,
                     parent_blocks: Vec<IPFSHash>,
                     text: String,
                     time: Option<NaiveDateTime>)
    -> impl Future<Item = IPFSHash, Error = Error>
{
    let client3 = client.clone();
    let client4 = client.clone();
    let client5 = client.clone();

    let iterator = parent_blocks
        .clone()
        .into_iter()
        .map(move |parent_block| {
            let client1 = client.clone();
            let client2 = client.clone();
            resolve_block(client1, &parent_block)
                .and_then(move |block| {
                    resolve_content(client2, block.content())
                })
                .map(|content| content.devices().to_vec())
        });

    ::futures::future::join_all(iterator)
        .and_then(move |devices| {
            let devices = Iterator::flatten(devices.into_iter()).unique().collect();

            put_plain(client3, text.into_bytes())
                .and_then(move |content_hash| {
                    let post = Payload::Post {
                        content_format: ::mime::TEXT_PLAIN.into(),
                        content: content_hash,
                        reply_to: None,

                        comments_will_be_propagated: None,
                        comments_propagated_until: None,
                    };

                    let ts          = time.map(Timestamp::from);
                    let content_obj = Content::new(devices, ts, post);

                    put_content(client4, &content_obj)
                })
        })
        .and_then(move |content_obj_hash| {
            let block = Block::new(protocol_version(), parent_blocks, content_obj_hash);
            put_block(client5, &block)
        })
}



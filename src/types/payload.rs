use std::collections::BTreeMap;

use anyhow::Error;

use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;
use crate::types::util::MimeType;
use crate::types::util::Timestamp;
use crate::repository::repository::Repository;


/// The Payload type represents the Payload of a Content object
///
/// The Payload type contains several variants, as an update (new block) may be either just a
/// metadata update (like a merge) or something more meaningful.
///
/// For example, the payload might be a `Payload::Post`, Alice has posted new kitten pictures!
/// Or a `Payload::ProfileUpdate` which contains information about Alice herself
///
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "payload_type")]
pub enum Payload {

    /// A variant that represents no payload
    ///
    /// This normally happens when merging two chains.
    None,

    /// A post
    ///
    /// A post can be anything which contains data. The data itself is put behind an IPFS hash, the
    /// Payload::Post only contains meta-information like payload size, content format (MIME) and
    /// optionally a list of IPFS hashes which are related to the post itself.
    Post {
        /// Format of the actual content
        #[serde(rename = "content-format")]
        content_format: MimeType,

        /// IPFS hash pointing to the actual content
        #[serde(rename = "content")]
        content: IPFSHash,

        /// If this post is a reply to another post, this field can be used to point to the
        /// replied-to post.
        #[serde(rename = "reply-to")]
        #[serde(default)]
        reply_to: Option<IPFSHash>,

        //
        //
        // Metadata for the post which should be visible to others
        //
        //

        /// A flag whether comments to this post will be propagated
        ///
        /// From a technical POV, comments can be done anyways, but the client which published the
        /// comment has to "accept" comments (via `PostComments`) to make them visible to others.
        /// This flag indicates whether the client will do that (either manually or automatically,
        /// which is not indicated here).
        ///
        /// # Available values
        ///
        /// A value of `Some(false)` means that comments will not be propagated, so others will not
        /// see them. A UI may not show "add comment"-buttons if this is set to `Some(false)`.
        ///
        /// A value of `Some(true)` means that comments will eventually be propagated. This means
        /// that the author might accept them by hand or tell their client to automatically accept
        /// all comments. This distinction is client-side implementation specific.
        ///
        /// A value of `None` indicates no setting here, which means that the client might or might
        /// not propagate any comments.
        #[serde(rename = "comments-will-be-propagated")]
        #[serde(default)]
        comments_will_be_propagated: Option<bool>,

        /// A value which describes until what date/time comments will be propagated
        ///
        /// This is a hint for other users whether comments will be propagated or not.
        /// A UI might not show a "Reply" button after that date.
        #[serde(rename = "comments-propagated-until")]
        #[serde(default)]
        comments_propagated_until: Option<Timestamp>,
    },

    /// Comments for a post
    ///
    /// Propagating answers to a post must be done by the author of the post itself.
    /// This variant is for publishing a message "These are the comments on my post <hash>".
    ///
    /// Always all comments should be published, not just the new ones!
    AttachedPostComments {
        /// The Hash of the Block for the Post which the comments are for
        #[serde(rename = "comments-for")]
        comments_for: IPFSHash,

        /// Hashes of direct answers to the post pointed to by "comments_for"
        /// This list always represents a full list of all answers. As comments are added, old
        /// versions of this object should be ignored by clients if newer variants for the same `comments_for`-object are published.
        #[serde(rename = "refs")]
        #[serde(default)]
        refs: Vec<IPFSHash>,
    },

    /// A variant describing a profile
    ///
    /// A profile contains the whole set of data which is considered "current" for the
    /// profile. Older versions of this shall then be ignored by clients.
    Profile {

        /// The self-assigned names of a user.
        #[serde(rename = "names")]
        names: Vec<String>,

        /// An optional user profile picture
        ///
        /// The picture itself is located behind a IPFS hash. If the hash does not resolve to a
        /// picture, clients should ignore it.
        ///
        /// A profile may only contain _one_ profile picture in the current version of the
        /// protocol.
        #[serde(rename = "picture")]
        #[serde(default)]
        picture: Option<IPFSHash>,

        /// A "more" field where arbitrary data can be stored. Like "Biography", "Hobbies",
        /// "Political opinion" or even pictures, ...
        ///
        /// The stored data can be of any type.
        #[serde(rename = "user-defined")]
        #[serde(default)]
        more: BTreeMap<String, Userdata>,
    },
}

impl Payload {
    /// Helper function
    pub fn is_none(&self) -> bool {
        match self {
            &Payload::None => true,
            _ => false,
        }
    }

    /// Helper function
    pub fn is_post(&self) -> bool {
        match self {
            &Payload::Post {..} => true,
            _ => false,
        }
    }

    /// Helper function
    pub fn is_attached_post_comments(&self) -> bool {
        match self {
            &Payload::AttachedPostComments {..} => true,
            _ => false,
        }
    }

    /// Helper function
    pub fn is_profile(&self) -> bool {
        match self {
            &Payload::Profile {..} => true,
            _ => false,
        }
    }

    pub async fn load(self, r: &Repository) -> Result<LoadedPayload, Error> {
        match self {
            Payload::None => Ok(LoadedPayload::None),
            Payload::Post {
                content_format,
                content,
                reply_to,
                comments_will_be_propagated,
                comments_propagated_until
            } => {
                Ok({
                    LoadedPayload::Post {
                        content_format,
                        content: r.get_raw_bytes(content).await?,
                        reply_to,
                        comments_will_be_propagated,
                        comments_propagated_until
                    }
                })
            },

            Payload::AttachedPostComments { comments_for, refs } => {
                Ok({
                    LoadedPayload::AttachedPostComments {
                        comments_for,
                        refs
                    }
                })
            },

            Payload::Profile { names, picture, more } => {
                let picture = if let Some(p) = picture {
                    Some(r.get_raw_bytes(p).await?)
                } else {
                    None
                };

                Ok({
                    LoadedPayload::Profile {
                        names,
                        picture,
                        more
                    }
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Userdata {
    #[serde(rename = "mimetype")]
    mimetype: MimeType,

    #[serde(rename = "data")]
    data: IPFSHash,
}

#[derive(Debug)]
pub enum LoadedPayload {
    None,

    Post {
        content_format: MimeType,
        content: Vec<u8>,
        reply_to: Option<IPFSHash>,
        comments_will_be_propagated: Option<bool>,
        comments_propagated_until: Option<Timestamp>,
    },

    AttachedPostComments {
        comments_for: IPFSHash,
        refs: Vec<IPFSHash>,
    },

    Profile {
        names: Vec<String>,
        picture: Option<Vec<u8>>,
        more: BTreeMap<String, Userdata>,
    },
}

impl LoadedPayload {
    pub fn is_none(&self) -> bool {
        match self {
            &LoadedPayload::None => true,
            _ => false,
        }
    }

    pub fn is_post(&self) -> bool {
        match self {
            &LoadedPayload::Post { .. }=> true,
            _ => false,
        }
    }

    pub fn is_attached_post_comments(&self) -> bool {
        match self {
            &LoadedPayload::AttachedPostComments { .. } => true,
            _ => false,
        }
    }

    pub fn is_profile(&self) -> bool {
        match self {
            &LoadedPayload::Profile { .. } => true,
            _ => false,
        }
    }
}

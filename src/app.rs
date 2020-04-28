use std::collections::BTreeMap;

use failure::Error;

use crate::types::util::IPFSHash;
use crate::types::util::MimeType;
use crate::types::util::IPNSHash;
use crate::types::block::Block;
use crate::repository::repository::Repository;
use crate::types::content::Content;
use crate::types::content::Payload;
use crate::types::util::Timestamp;
use crate::version::protocol_version;

pub struct App {
    repo: Repository,
    device_name: IPNSHash,
    publishing_key: String
}

impl App {

    pub fn load(device_name: IPNSHash, publishing_key: String, host: &str, port: u16) -> Result<Self, Error> {
        Repository::new(host, port).map(|repo| App { repo, device_name, publishing_key })
    }

    pub async fn new_profile(repo: Repository, publishing_key: &str, names: Vec<String>) -> Result<Self, Error> {
        let payload = Payload::Profile {
            names,
            picture: None,
            more: BTreeMap::new(),
        };
        let timestamp = Timestamp::now();
        let content = Content::new(vec![], Some(timestamp), payload);

        let content_hash = repo.put_content(content).await?;
        let head         = repo.put_block(Block::new(protocol_version(), vec![], content_hash)).await?;
        let device_name  = repo.publish(&publishing_key, &head).await?;

        Ok(App { repo, device_name, publishing_key: publishing_key.to_string() })
    }

    pub async fn new_post(&self,
                          content: Vec<u8>,
                          mime: MimeType,
                          reply_to: Option<IPFSHash>,
                          comments_will_be_propagated: Option<bool>,
                          comments_propagated_until: Option<Timestamp>)
        -> Result<(), Error>
    {
        let content_hash = self.repo.put_raw_bytes(content).await?;

        let payload = Payload::Post {
            content_format: mime,
            content: content_hash,
            reply_to,
            comments_will_be_propagated,
            comments_propagated_until,
        };
        let timestamp = Timestamp::now();
        let content = Content::new(vec![], Some(timestamp), payload);

        let content_hash = self.repo.put_content(content).await?;

        let head         = self.repo.put_block(Block::new(protocol_version(), vec![], content_hash)).await?;
        let device_name  = self.repo.publish(&self.publishing_key, &head).await?;

        Ok(())
    }


}


use std::collections::BTreeMap;

use failure::Error;

use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;
use crate::repository::repository::Repository;
use crate::types::content::Content;
use crate::types::content::Payload;
use crate::types::util::Timestamp;

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

        let head        = repo.put_content(content).await?;
        let device_name = repo.publish(&publishing_key, &head).await?;

        Ok(App { repo, device_name, publishing_key: publishing_key.to_string() })
    }


}


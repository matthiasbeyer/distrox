use std::collections::HashMap;

use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;

pub struct App {
    repo: Repository,
    device_name: String,
    publishing_key: String
}

impl App {

    pub fn load(device_name: String, publishing_key: String, host: &str, port: u16) -> Result<Self, Error> {
        Repository::new(host, port).map(|repo| App { repo, device_name, publishing_key })
    }

    pub async fn new_profile(repo: Repository, names: Vec<String>) -> Result<Self> {
        let payload = Payload::Profile {
            names,
            picture: None,
            more: BTreeMap::new(),
        };
        let timestamp = types::Timestamp::now();
        let content = Content::new(vec![], timestame, payload);

        let head        = repository.put_content(content).await?;
        let device_name = repository.publish(&publishing_key, &head).await?;

        Ok(App { repository, device_name, publishing_key })
    }


}


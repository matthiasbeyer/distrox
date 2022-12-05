use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use crate::{client::Client, error::Error};

pub struct Profile {
    client: Client,
    state_file_path: PathBuf,
    state: State,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    latest_node: Option<cid::Cid>,
    key_name: String,
    key_id: String, // TODO: Is this the right type for this job?
}

impl State {
    fn new(key_name: String, key_id: String) -> Self {
        Self {
            latest_node: None,
            key_name,
            key_id,
        }
    }
}

impl Profile {
    pub async fn create(
        backend_addr: std::net::SocketAddr,
        key_name: String,
        state_file_path: PathBuf,
    ) -> Result<Self, Error> {
        let (client, key) = Client::create(backend_addr, key_name).await?;

        let state = State::new(key.name().to_string(), key.id().to_string());
        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .create_new(true)
            .open(&state_file_path)
            .await?
            .write_all(toml::to_string_pretty(&state)?.as_bytes())
            .await?;

        Ok({
            Profile {
                client,
                state_file_path,
                state,
            }
        })
    }

    pub async fn load(
        backend_addr: std::net::SocketAddr,
        state_file_path: PathBuf,
    ) -> Result<Self, Error> {
        let state: State = tokio::fs::read_to_string(&state_file_path)
            .await
            .map_err(Error::from)
            .and_then(|s| toml::from_str(&s).map_err(Error::from))?;

        let client = Client::new(backend_addr)?;

        Ok({
            Profile {
                client,
                state_file_path,
                state,
            }
        })
    }

    pub async fn post_text(&mut self, text: String) -> Result<(), Error> {
        let text_cid = self.client.put_text(text).await?;
        let payload_cid = {
            let mime = mime::TEXT.to_string();
            let now = now();
            let payload = crate::types::Payload::new(mime, now, text_cid);
            self.client.put_payload(payload).await?
        };

        let node_cid = {
            let version = crate::API_VERSION.to_string();
            let parents = self.state.latest_node.into_iter().collect();
            let node = crate::types::Node::new(version, parents, payload_cid);
            self.client.put_node(node).await?
        };

        self.state.latest_node = Some(node_cid);
        Ok(())
    }
}

fn now() -> crate::types::DateTime {
    let now = time::OffsetDateTime::now_utc();
    crate::types::DateTime::from(now)
}

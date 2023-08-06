use std::io::Cursor;

use distrox_types::{
    post::{OriginalPost, Post},
    util::{Mime, OffsetDateTime},
};
use libp2p::Multiaddr;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    command::CommandReceiver, configuration::Configuration, error::Error, network::Network,
    state::State,
};

pub struct Application {
    app_state: Mutex<AppState>,

    network: Network,
}

impl Application {
    pub async fn load_from_xdg(xdg: xdg::BaseDirectories) -> Result<Self, Error> {
        let (config, state) = tokio::try_join!(
            Configuration::load_from_path(xdg.get_config_file("config.toml")),
            State::load_from_path(xdg.get_state_file("state.toml")),
        )?;

        let network = {
            let storage_path = config.network().storage_path().to_path_buf();
            let bootstrap = crate::network::BootstrapNodes(
                config
                    .network()
                    .bootstrap_nodes()
                    .iter()
                    .cloned()
                    .map(|n| n.try_into())
                    .collect::<Result<Vec<_>, Error>>()?,
            );
            let listening = crate::network::ListeningAddrs(
                config
                    .network()
                    .listening_addrs()
                    .iter()
                    .cloned()
                    .map(|n| n.try_into())
                    .collect::<Result<Vec<_>, Error>>()?,
            );

            Network::load(storage_path, bootstrap, listening).await?
        };

        let app_state = Mutex::new(AppState { config, state });
        Ok(Application { app_state, network })
    }

    pub async fn run(&self, mut receiver: CommandReceiver) -> Result<(), Error> {
        while let Some(command) = receiver.recv().await {
            match command {
                crate::command::Command::QuitApp => return Ok(()),
                crate::command::Command::PostText { text } => {
                    let latest_post = self.app_state.lock().await.get_latest_post()?;

                    let content_id = self
                        .network
                        .insert_blob(futures::stream::iter(text.bytes()))
                        .await?;

                    let new_post = Post::Original({
                        OriginalPost {
                            content: content_id,
                            content_mime: Mime(mime::TEXT_PLAIN_UTF_8),
                            timestamp: OffsetDateTime(time::OffsetDateTime::now_utc()),
                        }
                    });

                    let post_id = self.network.insert_post(new_post).await?;

                    let new_node = distrox_types::node::Node {
                        protocol_version: distrox_types::protocol::ProtocolVersion(0),
                        parents: latest_post.into_iter().collect(),
                        post: Some(post_id),
                    };

                    let node_id = self.network.insert_node(new_node).await?;

                    self.app_state.lock().await.set_latest_post(node_id).await?;
                }

                crate::command::Command::ConnectTo { uri } => {
                    let multiaddr: Multiaddr = uri.parse().unwrap();
                    info!(?uri, "Connecting");
                    let result = self.network.connect_without_peer(multiaddr).await;
                    info!(?uri, ?result, "Connecting finished");
                }
            }
        }

        Ok(())
    }
}

struct AppState {
    #[allow(unused)]
    config: Configuration,
    state: State,
}

impl AppState {
    fn get_latest_post(&self) -> Result<Option<cid::Cid>, Error> {
        self.state
            .latest_post()
            .map(|bytes| cid::Cid::read_bytes(Cursor::new(bytes)))
            .transpose()
            .map_err(Error::from)
    }

    async fn set_latest_post(&mut self, post: cid::Cid) -> Result<(), Error> {
        let bytes = post.to_bytes();
        self.state.store_latest_post(bytes).await
    }
}

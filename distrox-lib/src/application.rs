use crate::{configuration::Configuration, error::Error, network::Network, state::State};

pub struct Application {
    app_state: AppState,

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
                    .into_iter()
                    .cloned()
                    .map(|n| n.try_into())
                    .collect::<Result<Vec<_>, Error>>()?,
            );
            let listening = crate::network::ListeningAddrs(
                config
                    .network()
                    .listening_addrs()
                    .into_iter()
                    .cloned()
                    .map(|n| n.try_into())
                    .collect::<Result<Vec<_>, Error>>()?,
            );

            Network::load(storage_path, bootstrap, listening).await?
        };

        let app_state = AppState { config, state };
        Ok(Application { app_state, network })
    }
}

struct AppState {
    config: Configuration,
    state: State,
}

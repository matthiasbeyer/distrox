#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DistroxLib(#[from] distrox_lib::error::Error),

    #[error(transparent)]
    DistroxGui(#[from] distrox_gui::error::Error),

    #[error(transparent)]
    Xdg(#[from] xdg::BaseDirectoriesError),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
}

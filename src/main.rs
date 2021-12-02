use anyhow::Result;

pub mod cli;
pub mod client;
mod commands;
pub mod config;
pub mod consts;
pub mod ipfs_client;
pub mod profile;
pub mod types;
mod gui;

fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        Some(("profile", matches)) => crate::commands::profile(matches).await,
        Some(("gui", _)) => crate::gui::run(),
        Some((other, _)) => {
            log::error!("No subcommand {} implemented", other);
            Ok(())
        },

        _ => {
            log::error!("Don't know what to do");
            Ok(())
        },
    }
}


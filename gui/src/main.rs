use anyhow::Result;

mod cli;
mod gui;

use distrox_lib::*;

fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        None => crate::gui::run(),
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



use anyhow::Result;

mod cli;
mod gui;

use distrox_lib::*;

fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        None => {
            let name = matches.value_of("name").map(String::from).unwrap(); // safe by clap
            crate::gui::run(name)
        },
        Some((other, _)) => {
            log::error!("No subcommand {} implemented", other);
            Ok(())
        },
    }
}



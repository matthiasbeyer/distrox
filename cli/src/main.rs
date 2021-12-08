use anyhow::Result;

mod cli;
mod commands;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        Some(("profile", matches)) => crate::commands::profile(matches).await,
        Some(("gui", _)) => {
            unimplemented!()
        },
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



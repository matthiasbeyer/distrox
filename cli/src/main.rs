use anyhow::Result;

mod cli;
mod profile;
mod watch;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        Some(("profile", matches)) => crate::profile::profile(matches).await,
        Some(("watch", matches)) => crate::watch::watch(matches).await,
        Some(("gui", _)) => {
            unimplemented!()
        }
        Some((other, _)) => {
            log::error!("No subcommand {} implemented", other);
            Ok(())
        }

        _ => {
            log::error!("Don't know what to do");
            Ok(())
        }
    }
}

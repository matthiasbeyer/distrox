use anyhow::Result;

mod app;
mod cli;
mod gossip;
mod post;
mod timeline;

fn main() -> Result<()> {
    env_logger::try_init()?;
    let matches = crate::cli::app().get_matches();

    match matches.subcommand() {
        None => {
            let name = matches.value_of("name").map(String::from).unwrap(); // safe by clap
            crate::app::run(name)
        }
        Some((other, _)) => {
            log::error!("No subcommand {} implemented", other);
            Ok(())
        }
    }
}

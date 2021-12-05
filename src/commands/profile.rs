use anyhow::Result;
use clap::ArgMatches;

use crate::config::Config;
use crate::profile::Profile;

pub async fn profile(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("create", m)) => profile_create(m).await,
        _ => unimplemented!(),
    }
}

async fn profile_create(matches: &ArgMatches) -> Result<()> {
    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let state_dir = Profile::state_dir_path(&name)?;
    log::info!("Creating '{}' in {}", name, state_dir.display());

    let profile = Profile::create(&state_dir, &name, Config::default()).await?;
    log::info!("Saving...");
    profile.save().await
}

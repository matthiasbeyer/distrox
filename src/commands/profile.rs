use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use clap::ArgMatches;

use crate::config::Config;
use crate::profile::Profile;

pub async fn profile(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("create", m)) => profile_create(m).await,
        Some(("serve", m)) => profile_serve(m).await,
        _ => unimplemented!(),
    }
}

async fn profile_create(matches: &ArgMatches) -> Result<()> {
    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let state_dir = Profile::state_dir_path(&name)?;
    log::info!("Creating '{}' in {}", name, state_dir.display());

    let profile = Profile::create(&state_dir, &name, Config::default()).await?;
    log::info!("Saving...");
    profile.save().await?;

    log::info!("Shutting down...");
    profile.exit().await
}

async fn profile_serve(matches: &ArgMatches) -> Result<()> {
    use ipfs::MultiaddrWithPeerId;

    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let connect_peer = matches.value_of("connect").map(|s| {
        s.parse::<MultiaddrWithPeerId>()
            .map_err(anyhow::Error::from)
    }).transpose()?;

    let state_dir = Profile::state_dir_path(&name)?;

    log::info!("Loading '{}' from {}", name, state_dir.display());
    let profile = Profile::load(Config::default(), &name).await?;
    log::info!("Profile loaded");
    log::info!("Profile HEAD = {:?}", profile.head());

    if let Some(connect_to) = connect_peer {
        log::info!("Connecting to {:?}", connect_to);
        profile.connect(connect_to).await?;
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).context("Error setting Ctrl-C handler")?;

    log::info!("Serving...");
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await // sleep not so busy
    }
    log::info!("Shutting down...");
    profile.exit().await
}

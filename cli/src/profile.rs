use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use clap::ArgMatches;

use distrox_lib::profile::Profile;
use distrox_lib::types::Payload;

pub async fn profile(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("create", m)) => profile_create(m).await,
        Some(("serve", m)) => profile_serve(m).await,
        Some(("post", m)) => profile_post(m).await,
        Some(("cat", m)) => profile_cat(m).await,
        _ => unimplemented!(),
    }
}

async fn profile_create(matches: &ArgMatches) -> Result<()> {
    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let state_dir = Profile::state_dir_path(&name)?;
    log::info!("Creating '{}' in {}", name, state_dir.display());

    let profile = Profile::create(&state_dir, &name).await?;
    log::info!("Saving...");
    profile.save().await?;

    log::info!("Shutting down...");
    profile.exit().await
}

async fn profile_serve(matches: &ArgMatches) -> Result<()> {
    use ipfs::MultiaddrWithPeerId;

    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let listen_addrs = matches.values_of("listen")
        .map(|v| {
            v.map(|s| s.parse::<ipfs::Multiaddr>().map_err(anyhow::Error::from))
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;
    let connect_peer = matches.values_of("connect")
        .map(|v| {
            v.map(|s| {
                s.parse::<MultiaddrWithPeerId>().map_err(anyhow::Error::from)
            })
            .collect::<Result<Vec<_>>>()
        })
        .transpose()?;

    let state_dir = Profile::state_dir_path(&name)?;

    log::info!("Loading '{}' from {}", name, state_dir.display());
    let profile = Profile::load(&name).await?;
    log::info!("Profile loaded");
    if let Some(head) = profile.head().as_ref() {
        log::info!("Profile HEAD = {}", head);
    }

    if let Some(listen) = listen_addrs {
        for l in listen {
            log::debug!("Adding listening address: {}", l);
            profile.listen_on(l).await?;
        }
    }

    {
        let addrs = profile.client().own_addresses().await?;
        if addrs.is_empty() {
            log::error!("No own address");
        } else {
            for addr in addrs {
                log::info!("Own addr: {}", addr);
            }
        }
    }

    if let Some(connect_to) = connect_peer {
        for c in connect_to {
            log::info!("Connecting to {:?}", c);
            profile.connect(c).await?;
        }
    }

    let mut gossip_channel = Box::pin({
        profile.client()
            .pubsub_subscribe("distrox".to_string())
            .await
            .map(|stream| {
                use distrox_lib::gossip::deserializer::GossipDeserializer;
                use distrox_lib::gossip::deserializer::LogStrategy;

                GossipDeserializer::<LogStrategy>::new().run(stream)
            })?
    });

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let own_peer_id = profile.client().own_id().await?;

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).context("Error setting Ctrl-C handler")?;

    log::info!("Serving...");
    while running.load(Ordering::SeqCst) {
        use futures::stream::StreamExt;
        use distrox_lib::gossip::GossipMessage;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await; // sleep not so busy

        tokio::select! {
            own = profile.gossip_own_state("distrox".to_string()) => own?,
            other = gossip_channel.next() => {
                let gossip_myself = other.as_ref().map(|(source, _)| *source == own_peer_id).unwrap_or(false);

                if !gossip_myself {
                    log::trace!("Received gossip: {:?}", other);
                }
            }
        }
    }
    log::info!("Shutting down...");
    profile.exit().await
}

async fn profile_post(matches: &ArgMatches) -> Result<()> {
    let text = match matches.value_of("text") {
        Some(text) => String::from(text),
        None => if matches.is_present("editor") {
            editor_input::input_from_editor("")?
        } else {
            unreachable!()
        }
    };

    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let state_dir = Profile::state_dir_path(&name)?;
    log::info!("Creating '{}' in {}", name, state_dir.display());

    log::info!("Loading '{}' from {}", name, state_dir.display());
    let mut profile = Profile::load(&name).await?;
    log::info!("Profile loaded");
    log::info!("Profile HEAD = {:?}", profile.head());

    log::info!("Posting text...");
    profile.post_text(text).await?;
    log::info!("Posting text finished");
    profile.save().await?;
    log::info!("Saving profile state to disk finished");
    profile.exit().await
}

async fn profile_cat(matches: &ArgMatches) -> Result<()> {
    use distrox_lib::stream::NodeStreamBuilder;
    use futures::stream::StreamExt;

    let name = matches.value_of("name").map(String::from).unwrap(); // required
    let state_dir = Profile::state_dir_path(&name)?;
    log::info!("Creating '{}' in {}", name, state_dir.display());

    log::info!("Loading '{}' from {}", name, state_dir.display());
    let profile = Profile::load(&name).await?;
    log::info!("Profile loaded");
    if let Some(head) = profile.head() {
        log::info!("Profile HEAD = {:?}", head);
        NodeStreamBuilder::starting_from(head.clone())
            .into_stream(profile.client().clone())
            .then(|node| async {
                match node {
                    Err(e) => Err(e),
                    Ok(node) => {
                        profile.client()
                            .get_payload(node.payload())
                            .await
                    }
                }
            })
            .then(|payload| async {
                match payload {
                    Err(e) => Err(e),
                    Ok(payload) => {
                        profile.client()
                            .get_content_text(payload.content())
                            .await
                            .map(|text| (payload, text))
                    }
                }
            })
            .then(|res| async {
                use std::io::Write;
                match res {
                    Err(e) => {
                        let out = std::io::stderr();
                        let mut lock = out.lock();
                        writeln!(lock, "Error: {:?}", e)?;
                    }
                    Ok((payload, text)) => {
                        let out = std::io::stdout();
                        let mut lock = out.lock();
                        writeln!(lock, "{time} - {cid}",
                            time = payload.timestamp().inner(),
                            cid = payload.content())?;

                        writeln!(lock, "{text}", text = text)?;
                        writeln!(lock, "")?;
                    },
                }
                Ok(())
            })
            .collect::<Vec<Result<()>>>()
            .await
            .into_iter()
            .collect::<Result<()>>()?;
    } else {
        eprintln!("Profile has no posts");
    }

    Ok(())
}

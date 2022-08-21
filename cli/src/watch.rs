use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use clap::ArgMatches;

use distrox_lib::client::Client;

pub async fn watch(args: &ArgMatches) -> Result<()> {
    let ipfs = boot_ipfs(
        args.value_of("state_dir")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("No state dir"))?,
    )
    .await?;
    let client = Client::new(ipfs);

    let own_peer_id = client.own_id().await?;
    log::info!("Own id = {}", own_peer_id);
    log::info!(
        "Own addresses = {}",
        client
            .own_addresses()
            .await?
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );

    let connect_addrs = args
        .values_of("connect")
        .map(|vals| {
            vals.map(|c| ipfs::MultiaddrWithPeerId::from_str(c).map_err(anyhow::Error::from))
                .collect::<Result<Vec<ipfs::MultiaddrWithPeerId>>>()
        })
        .transpose()?
        .unwrap_or_default();

    for addr in connect_addrs {
        log::info!("Connecting to {}", addr);
        client.connect(addr).await?
    }

    let mut gossip_channel = Box::pin({
        client
            .pubsub_subscribe("distrox".to_string())
            .await
            .map(|stream| {
                use distrox_lib::gossip::GossipDeserializer;
                use distrox_lib::gossip::LogStrategy;

                GossipDeserializer::<LogStrategy>::default().run(stream)
            })?
    });

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;

    log::info!("Serving...");
    loop {
        use futures::stream::StreamExt;

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let next = gossip_channel.next().await;
        let gossip_myself = next
            .as_ref()
            .map(|(source, _)| *source == own_peer_id)
            .unwrap_or(false);

        if !gossip_myself {
            log::info!("Received gossip: {:?}", next);
        }
    }
    log::info!("Shutting down...");

    client.exit().await
}

async fn boot_ipfs(state_dir: PathBuf) -> Result<distrox_lib::ipfs_client::IpfsClient> {
    let bootstrap = vec![]; // TODO
    let mdns = true; // TODO
    let keypair = ipfs::Keypair::generate_ed25519();

    let options = ipfs::IpfsOptions {
        ipfs_path: state_dir,
        keypair,
        bootstrap,
        mdns,
        kad_protocol: None,
        listening_addrs: vec![],
        span: Some(tracing::trace_span!("distrox-ipfs")),
    };

    let (ipfs, fut): (ipfs::Ipfs<_>, _) =
        ipfs::UninitializedIpfs::<_>::new(options).start().await?;
    tokio::task::spawn(fut);

    Ok(ipfs)
}

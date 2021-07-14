use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Result;
use daglib::DagBackend;
use rand_os::OsRng;
use rand_core::CryptoRng;
use rand_core::RngCore;
use ed25519_dalek::Keypair;
use ed25519_dalek::Signature;
use futures::stream::StreamExt;

extern crate clap_v3 as clap;

mod backend;
mod cli;
mod consts;
mod profile;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let app = crate::cli::app();

    log::info!("Booting backend");
    let mut backend = {
        // Testing configuration for the IPFS node in the backend.

        let tmp = PathBuf::from("/tmp/distrox.tmp");
        let sconf = ipfs_embed::StorageConfig {
            path: Some(tmp),
            cache_size_blocks: 100_000, // blocks kepts before GC
            cache_size_bytes: 1024 * 1024 * 1024, // 1GB before GC
            gc_interval: std::time::Duration::from_secs(60 * 60), // hourly
            gc_min_blocks: 0,
            gc_target_duration: std::time::Duration::from_secs(60), // 1 minute
        };

        let mut csprng = OsRng{};
        let nconf = ipfs_embed::NetworkConfig {
            node_name: String::from("distrox-devel"),
            node_key: ipfs_embed::Keypair::generate(&mut csprng),

            quic: ipfs_embed::TransportConfig::default(),
            psk: None,
            dns: None,
            mdns: None,
            kad: None,
            ping: None,
            identify: None,
            gossipsub: None,
            broadcast: None,
            bitswap: None,

        };

        let ipfs_configuration = ipfs_embed::Config {
            storage: sconf,
            network: nconf,
        };
        crate::backend::IpfsEmbedBackend::new_with_config(ipfs_configuration).await?
    };
    log::info!("Backend booted successfully");

    backend.ipfs()
        .listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?
        .next()
        .await
        .unwrap();

    match app.get_matches().subcommand() {
        ("create-profile", Some(mtch)) => {
            log::info!("Creating profile");
            let payload = mtch
                .value_of("content")
                .map(String::from)
                .map(crate::backend::Payload::now_from_text)
                .unwrap(); // Safe by clap

            let payload_cid = backend.write_payload(&payload).await?;
            log::info!("Payload written as {}", payload_cid);
            let node = crate::backend::Node::new(crate::consts::v1(), vec![], payload_cid);

            log::info!("Writing node for payload");
            let id = backend.put(node).await?;

            println!("id = {}", id.as_ref());
            log::info!("Creating profile finished");
            Ok(())
        },

        ("post", Some(mtch)) => {
            let payload = mtch
                .value_of("content")
                .map(String::from)
                .map(crate::backend::Payload::now_from_text)
                .unwrap(); // Safe by clap
            let parent = mtch
                .value_of("head")
                .map(cid::Cid::from_str)
                .transpose()?
                .map(crate::backend::Id::from)
                .unwrap(); // Safe by clap

            let payload_cid = backend.write_payload(&payload).await?;
            let node = crate::backend::Node::new(crate::consts::v1(), vec![parent], payload_cid);

            let id = backend.put(node).await?;

            println!("id = {}", id.as_ref());
            Ok(())
        },

        ("get", Some(mtch)) => {
            let head = mtch
                .value_of("head")
                .map(cid::Cid::from_str)
                .transpose()?
                .map(crate::backend::Id::from)
                .unwrap(); // Safe by clap

            log::info!("Fetching {} from backend", head.as_ref());
            let (id, node) = backend
                .get(head.clone())
                .await?
                .ok_or_else(|| anyhow!("Not found: {:?}", head))?;

            log::info!("Fetching payload of {} ({}) from backend", head.as_ref(), node.payload_id());
            let payload = backend.ipfs().fetch(node.payload_id(), backend.ipfs().peers()).await?;
            log::debug!("Fetched payload, now decoding...");
            let payload = payload.decode::<libipld::cbor::DagCborCodec, crate::backend::Payload>()?;
            log::debug!("Decoding successful");

            println!("id      = {}", id.as_ref());
            println!("node    = {:?}", node);
            println!("payload = {:?}", payload);
            Ok(())
        },

        (other, _) => {
            unimplemented!()
        },
    }
}


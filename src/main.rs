use std::path::PathBuf;

use anyhow::Result;
use daglib::DagBackend;

extern crate clap_v3 as clap;

mod backend;
mod cli;
mod consts;
mod profile;

#[tokio::main]
async fn main() -> Result<()> {
    let app = crate::cli::app();

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

        let nconf = ipfs_embed::NetworkConfig {
            node_key: libp2p_core::identity::Keypair::generate_ed25519(),
            node_name: String::from("distrox-devel"),
            enable_mdns: false, // don't know what this is, yet
            enable_kad: false, // don't know what this is, yet
            allow_non_globals_in_dht: false, // don't know what this is, yet
            psk: None, // Pre shared key for pnet.
            ping: libp2p_ping::PingConfig::new(), // Ping config.
            gossipsub: libp2p_gossipsub::GossipsubConfig::default(), // Gossipsub config.
            bitswap: ipfs_embed::BitswapConfig::new(), // Bitswap config.
        };

        let ipfs_configuration = ipfs_embed::Config {
            storage: sconf,
            network: nconf,
        };
        crate::backend::IpfsEmbedBackend::new_with_config(ipfs_configuration).await?
    };

    backend.ipfs().listen_on("/ip4/127.0.0.1/tcp/0".parse()?).await?;

    match app.get_matches().subcommand() {
        ("create-profile", Some(mtch)) => {
            let payload = mtch
                .value_of("content")
                .map(String::from)
                .map(crate::backend::Payload::now_from_text)
                .unwrap(); // Safe by clap

            let payload_cid = backend.write_payload(&payload).await?;
            let node = crate::backend::Node::new(crate::consts::v1(), vec![], payload_cid);

            let id = backend.put(node).await?;

            println!("id = {:?}", id);
            Ok(())
        },

        ("post", Some(mtch)) => {
            unimplemented!()
        },

        (other, _) => {
            unimplemented!()
        },
    }
}


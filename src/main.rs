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

pub mod cli;
pub mod client;
pub mod config;
pub mod consts;
pub mod ipfs_client;
pub mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let app = crate::cli::app();
    Ok(())
}


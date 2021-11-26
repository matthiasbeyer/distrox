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

mod cli;
mod consts;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let app = crate::cli::app();
    Ok(())
}


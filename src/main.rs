use anyhow::Result;

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


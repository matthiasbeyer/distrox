use anyhow::Result;

pub mod cli;
pub mod client;
pub mod config;
pub mod consts;
pub mod ipfs_client;
pub mod profile;
pub mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init()?;
    let _ = crate::cli::app();
    Ok(())
}


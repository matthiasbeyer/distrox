use std::path::PathBuf;

use structopt::StructOpt;
use anyhow::Error;

#[derive(Debug, StructOpt)]
#[structopt(name = "distrox", about = "Distrox - The distributed social network")]
pub struct CLI {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long)]
    trace: bool,

    #[structopt(parse(from_os_str))]
    configfile: Option<PathBuf>,
}

pub fn cli() -> Result<CLI, Error> {
    Ok(CLI::from_args())
}

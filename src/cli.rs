use std::path::PathBuf;

use structopt::StructOpt;
use failure::Error;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
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

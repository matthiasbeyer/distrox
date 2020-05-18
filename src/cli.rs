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

    #[structopt(short, long)]
    port: Option<u16>,

    #[structopt(parse(from_os_str))]
    configfile: Option<PathBuf>,

    #[structopt(subcommand)]
    cmd: Option<Command>
}

impl CLI {
    pub fn cmd(&self) -> Option<&Command> {
        self.cmd.as_ref()
    }

    pub fn port(&self) -> Option<u16> {
        self.port.as_ref().map(|p| *p)
    }
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "Start the server part (running in foreground")]
pub enum Command {
    Server,
}

pub fn cli() -> Result<CLI, Error> {
    Ok(CLI::from_args())
}

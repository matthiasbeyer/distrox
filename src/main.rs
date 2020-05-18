#![allow(warnings)]

extern crate ipfs_api;
extern crate chrono;
extern crate mime;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate clap;
extern crate toml;
extern crate config;
extern crate hyper;
extern crate env_logger;
extern crate itertools;
extern crate xdg;
extern crate handlebars;
extern crate web_view;
extern crate actix_rt;
extern crate actix_web;
extern crate failure;
extern crate pidlock;

#[macro_use] extern crate anyhow;
#[macro_use] extern crate is_match;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate tokio;
#[macro_use] extern crate add_getters_setters;
#[macro_use] extern crate structopt;

mod app;
mod cli;
mod configuration;
mod gui;
mod repository;
mod server;
mod types;
mod version;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::ops::Deref;
use std::sync::Arc;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use futures::future::Future;
use futures::future::FutureExt;
use futures::future::TryFutureExt;
use serde_json::to_string_pretty as serde_json_to_string_pretty;
use serde_json::from_str as serde_json_from_str;
use anyhow::Result;
use anyhow::Error;
use env_logger::Env;

use crate::app::App;
use crate::cli::*;
use crate::configuration::Configuration;
use crate::repository::repository::Repository;
use crate::types::block::Block;
use crate::types::content::Content;
use crate::types::payload::Payload;
use crate::types::util::IPFSHash;
use crate::types::util::IPNSHash;
use crate::types::util::MimeType;
use crate::types::util::Timestamp;
use crate::types::util::Version;

use std::process::exit;

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = cli()?;
    let _ = env_logger::from_env(Env::default().default_filter_or("info")).init();
    debug!("Logger initialized");

    let config_file_name = PathBuf::from("distrox.toml");
    let config: Configuration = {
        let configfile = xdg::BaseDirectories::with_prefix("distrox")?
            .find_config_file(&config_file_name)
            .ok_or_else(|| anyhow!("No configuration found"))?;

        let configstr = ::std::fs::read_to_string(&configfile)?;
        ::toml::from_str(&configstr)?
    };

    let port = cli.port().unwrap_or_else(|| *config.get_app_port());
    let adr = format!("127.0.0.1:{}", port);

    let mut server_lock = crate::server::mk_lock();
    let server_running  = crate::server::is_running(&server_lock);
    let start_server    = crate::server::do_start(&cli);

    match (server_running, start_server) {
        (true, false) => crate::gui::run_gui(config, adr),
        (false, false) => {
            // fork()
            let path = std::env::current_exe()?;
            let mut child = std::process::Command::new(path).arg("server").spawn()?;
            let r = crate::gui::run_gui(config, adr);
            child.kill()?;
            r
        },

        (false, true) => crate::server::run_server(server_lock, adr).await,

        (true, true) => {
            info!("Server is already running. Doing nothing.");
            return Ok(())
        },
    }
}


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
mod repository;
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
use actix_web::{web, HttpResponse, Responder};

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

fn start_server(cli: &CLI) -> bool {
    cli.cmd().map(|cmd| Command::Server == *cmd).unwrap_or(false)
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = cli()?;
    let _ = env_logger::init();
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

    if start_server(&cli) {
        let mut lock = pidlock::Pidlock::new("/tmp/distrox_server.pid");
        if *lock.state() == pidlock::PidlockState::Acquired {
            // We assume that the server is already running
            return Ok(())
        }

        let _ = lock.acquire().map_err(|_| anyhow!("Error while getting the PID lock"))?;
        actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .service(actix_web::web::resource("/{name}/{id}/index.html").to(index))
        })
        .bind(adr.clone())
        .expect(&format!("Could not bind to address {}", adr))
        .run()
        .await;

        lock.release().map_err(|_| anyhow!("Error while releasing the PID lock"))
    } else {
        let app = {
            let device_name = config.get_device_name();
            let device_key  = config.get_device_key();

            if let (Some(name), Some(key)) = (device_name, device_key) {
                let name        = IPNSHash::from(name.clone());
                let key         = key.clone();
                let api_url     = config.get_api_url().clone();
                let api_port    = config.get_api_port().clone();

                App::load(name, key, &api_url, api_port)
            } else {
                // ask user for name(s)
                // boot repository
                // load App object
                unimplemented!()
            }
        };

        let webview_content = web_view::Content::Url(adr.clone());

        web_view::builder()
            .title("My Project")
            .content(webview_content)
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .build()
            .map_err(Error::from)?
            .run()
            .map_err(Error::from)
    }
}

async fn index() -> impl Responder {
   HttpResponse::Ok()
}


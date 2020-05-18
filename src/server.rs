use std::path::PathBuf;

use anyhow::Error;
use anyhow::Result;
use pidlock::{Pidlock, PidlockState};
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::http::StatusCode;
use actix_web::body::Body;

use crate::app::App;
use crate::cli::*;
use crate::configuration::Configuration;
use crate::types::util::*;

pub fn mk_lock() -> Pidlock {
    pidlock::Pidlock::new("/tmp/distrox_server.pid")
}

pub fn do_start(cli: &CLI) -> bool {
    cli.cmd().map(|cmd| Command::Server == *cmd).unwrap_or(false)
}

pub fn is_running(server_lock: &Pidlock) -> bool {
    PathBuf::from("/tmp/distrox_server.pid").exists()
}


pub async fn run_server(config: Configuration, mut server_lock: Pidlock, adr: String) -> Result<()> {
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
    }?;

    info!("Starting server");
    let _ = server_lock.acquire().map_err(|_| anyhow!("Error while getting the PID lock"))?;

    info!("Got PID lock for server");
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("*", actix_web::web::get().to(index))
    })
    .bind(adr.clone())
    .expect(&format!("Could not bind to address {}", adr))
    .run()
    .await;

    info!("Server shutdown");
    info!("Releasing PID lock for server");
    server_lock.release().map_err(|_| anyhow!("Error while releasing the PID lock"))
}

async fn index() -> impl Responder {
    debug!("serve index");
    let s = format!("{pre}{style}{index}{post}",
        pre   = include_str!("../assets/index_pre.html"),
        style = include_str!("../assets/style.css"),
        index = include_str!("../assets/index.html"),
        post  = include_str!("../assets/index_post.html"),
    );

    HttpResponse::build(StatusCode::OK).body(Body::from_slice(s.as_bytes()))
}


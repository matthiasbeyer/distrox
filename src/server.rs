use actix_web::{web, HttpResponse, Responder};
use anyhow::Error;
use anyhow::Result;
use pidlock::{Pidlock, PidlockState};

use crate::cli::*;
use crate::types::util::*;

pub fn mk_lock() -> Pidlock {
    pidlock::Pidlock::new("/tmp/distrox_server.pid")
}

pub fn do_start(cli: &CLI) -> bool {
    cli.cmd().map(|cmd| Command::Server == *cmd).unwrap_or(false)
}

pub fn is_running(server_lock: &Pidlock) -> bool {
    *server_lock.state() == PidlockState::Acquired
}


pub async fn run_server(mut server_lock: Pidlock, adr: String) -> Result<()> {
    info!("Starting server");
    let _ = server_lock.acquire().map_err(|_| anyhow!("Error while getting the PID lock"))?;

    info!("Got PID lock for server");
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .service(actix_web::web::resource("/{name}/{id}/index.html").to(index))
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
   HttpResponse::Ok()
}


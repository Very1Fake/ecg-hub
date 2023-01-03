pub mod config;
pub mod error;

use std::net::SocketAddr;

use axum::{routing::get, Json, Router, Server};
use tracing::info;

use common::hub::HubStatus;
use config::{STATUS, Config};
use error::Error;

/// ECG Hub entrypoint
pub async fn app(config: Config) -> Result<(), Error> {
    let router = Router::new().route("/status", get(info));
    let addr = SocketAddr::new(config.addr.parse()?, config.port);

    info!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

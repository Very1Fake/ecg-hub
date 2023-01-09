use std::net::SocketAddr;

use axum::Server;
use sqlx::PgPool;

use crate::{app::HubState, config::Config, error::Error};

pub mod app;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod types;

pub type DB = PgPool;

/// ECG Hub entrypoint
pub async fn run(config: &Config) -> Result<(), Error> {
    let addr = SocketAddr::new(config.addr.parse()?, config.port);
    let router = HubState::new(config).await?.build_router();

    tracing::info!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

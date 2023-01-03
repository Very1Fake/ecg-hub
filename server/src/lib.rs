pub mod config;

use axum::{routing::get, Json, Router, Server};
use common::hub::HubStatus;
use config::STATUS;

/// ECG Hub entrypoint
pub async fn app() -> Result<(), hyper::Error> {
    let router = Router::new().route("/status", get(info));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

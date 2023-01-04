pub mod config;
pub mod error;
pub mod models;

use std::net::SocketAddr;

use axum::{extract::Query, routing::get, Json, Router, Server};
use chrono::Utc;
use hyper::StatusCode;
use models::queries::UserIdQuery;
use tracing::info;

use common::{
    hub::HubStatus,
    user::{UserData, UserStatus},
};
use config::{Config, STATUS};
use error::Error;
use uuid::Uuid;

use crate::models::entities::User;

/// ECG Hub entrypoint
pub async fn app(config: Config) -> Result<(), Error> {
    let addr = SocketAddr::new(config.addr.parse()?, config.port);
    let router = Router::new()
        .route("/status", get(info))
        .route("/user", get(user_get));

    info!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

async fn user_get(user_id: Query<UserIdQuery>) -> Result<Json<UserData>, StatusCode> {
    let user = User {
        uuid: Uuid::nil(),
        username: String::from("very1fake"),
        email: String::from("very1fake.coder@gmail.com"),
        password: String::new(),
        status: UserStatus::Active,
        updated: Utc::now(),
        created: Utc::now(),
    };

    Ok(Json(
        match (user_id.uuid, &user_id.username) {
            (Some(uuid), _) => {
                if uuid.is_nil() {
                    user
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            (None, Some(username)) => {
                if username == "very1fake" {
                    user
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        }
        .into(),
    ))
}

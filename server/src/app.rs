use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{routing::get, Router, Server};
use chrono::Utc;
use common::user::UserStatus;
use uuid::Uuid;

use crate::{
    config::Config,
    error::Error,
    handlers::{info, user_get},
    models::entities::UserEntity,
};

#[derive(Debug)]
pub struct HubState {
    pub users: HashMap<Uuid, UserEntity>,
}

impl HubState {
    pub fn new() -> Self {
        Self {
            users: [(
                Uuid::nil(),
                UserEntity {
                    uuid: Uuid::nil(),
                    username: String::from("very1fake"),
                    email: String::from("very1fake.coder@gmail.com"),
                    password: String::new(),
                    status: UserStatus::Active,
                    updated: Utc::now(),
                    created: Utc::now(),
                },
            )]
            .into(),
        }
    }

    pub fn build_router(self) -> Router {
        Router::new()
            .route("/status", get(info))
            .route("/user", get(user_get))
            .with_state(Arc::new(self))
    }
}

/// ECG Hub entrypoint
pub async fn start(config: Config, router: Router) -> Result<(), Error> {
    let addr = SocketAddr::new(config.addr.parse()?, config.port);

    tracing::info!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

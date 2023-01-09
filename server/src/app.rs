use std::{sync::Arc, time::Duration};

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;

use crate::{
    config::Config,
    error::Error,
    handlers::{info, user_get, user_post},
    DB,
};

#[derive(Debug)]
pub struct HubState {
    pub db: DB,
}

impl HubState {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let db = PgPoolOptions::new()
            .min_connections(config.db_pool_min)
            .max_connections(config.db_pool_max)
            .acquire_timeout(Duration::from_secs(8))
            .connect(&config.db_uri())
            .await?;

        Ok(Self { db })
    }

    pub fn build_router(self) -> Router {
        Router::new()
            .route("/status", get(info))
            .route("/user", get(user_get).post(user_post))
            .with_state(Arc::new(self))
    }
}

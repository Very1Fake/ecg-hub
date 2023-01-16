use std::{sync::Arc, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;

use crate::{
    config::Config,
    error::Error,
    handlers::{
        auth_login, info, pubkey, token_refresh, token_revoke, token_revoke_all, user_get,
        user_post,
    },
    keys::Keys,
    DB,
};

pub struct HubState {
    pub keys: Keys,
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

        Ok(Self {
            keys: config.keys(),
            db,
        })
    }

    pub fn build_router(self) -> Router {
        Router::new()
            .route("/status", get(info))
            .route("/user", get(user_get).post(user_post))
            .route("/pubkey", get(pubkey))
            .route("/auth/login", post(auth_login))
            .route("/token/refresh", get(token_refresh))
            .route("/token/revoke", get(token_revoke))
            .route("/token/revoke_all", get(token_revoke_all))
            .with_state(Arc::new(self))
    }
}

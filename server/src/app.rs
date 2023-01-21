use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};
use axum_server::{
    accept::DefaultAcceptor,
    bind, bind_rustls,
    tls_rustls::{RustlsAcceptor, RustlsConfig},
    Server,
};
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::{
    config::Config,
    error::Error,
    handlers::{
        auth_login, health, info, pubkey, token_refresh, token_revoke, token_revoke_all, user_get,
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
            .acquire_timeout(Duration::from_secs(config.db_timeout))
            .connect(&config.db_uri())
            .await?;

        Ok(Self {
            keys: config.keys(),
            db,
        })
    }

    pub fn build_router(self) -> Router {
        Router::new()
            .route("/health", get(health))
            .route("/info", get(info))
            .route("/user", get(user_get).post(user_post))
            .route("/pubkey", get(pubkey))
            .route("/auth/login", post(auth_login))
            .route("/token/refresh", get(token_refresh))
            .route("/token/revoke", get(token_revoke))
            .route("/token/revoke_all", get(token_revoke_all))
            .with_state(Arc::new(self))
    }
}

pub enum ServerMode {
    Https(Server<RustlsAcceptor>),
    Http(Server<DefaultAcceptor>),
}

/// ECG Hub entrypoint
pub async fn run(config: &Config) -> Result<(), Error> {
    let addr = SocketAddr::new(config.addr.parse()?, config.port);
    let router = HubState::new(config).await?.build_router();

    let server = if let (Some(cert), Some(key)) = (&config.ssl_cert, &config.ssl_key) {
        let tls = RustlsConfig::from_pem_file(cert, key).await?;
        info!("Starting HTTPS server");
        ServerMode::Https(bind_rustls(addr, tls))
    } else {
        info!("Starting HTTP server");
        ServerMode::Http(bind(addr))
    };

    info!("Listening on {}", addr);

    match server {
        ServerMode::Https(https) => https.serve(router.into_make_service()).await,
        ServerMode::Http(http) => http.serve(router.into_make_service()).await,
    }?;

    Ok(())
}

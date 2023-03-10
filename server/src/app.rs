use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    routing::{get, post, put},
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
        health, pubkey, status, token_pit, token_refresh, token_revoke, token_revoke_all,
        user_data, user_info, user_login, user_password, user_register, user_sessions,
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
            .route("/", get(status))
            .route("/status", get(status))
            .route("/health", get(health))
            .route("/pubkey", get(pubkey))
            .route("/user/info", get(user_info))
            .route("/user/data", get(user_data))
            .route("/user/login", post(user_login))
            .route("/user/register", post(user_register))
            .route("/user/password", put(user_password))
            .route("/user/sessions", get(user_sessions))
            .route("/token/refresh", get(token_refresh))
            .route("/token/revoke", get(token_revoke))
            .route("/token/revoke_all", get(token_revoke_all))
            .route("/token/pit", get(token_pit))
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

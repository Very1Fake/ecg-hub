use ecg_hub_server::{
    app::{start, HubState},
    config::Config,
    error::Error,
};
use tokio::runtime::Builder;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Error> {
    // Parse config from env
    let config: Config = envy::prefixed("HUB_").from_env()?;

    // Start logger
    tracing_subscriber::registry()
        .with(config.log_filter())
        .with(fmt::layer())
        .init();

    // Run the server
    Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
        .block_on(start(config, HubState::new().build_router()))?;

    Ok(())
}

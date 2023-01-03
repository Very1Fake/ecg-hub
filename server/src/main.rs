use ecg_hub_server::{app, error::Error};
use tokio::runtime::Builder;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<(), Error> {
    // Parse config from env
    let config = envy::prefixed("HUB_").from_env()?;

    // Start logger
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "ecg_hub_server=debug".into()))
        .with(fmt::layer())
        .init();    

    // Run the server
    Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
        .block_on(app(config))?;

    Ok(())
}

use ecg_hub_server::{config::Config, error::Error, run};
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
        .enable_time()
        .build()
        .unwrap()
        .block_on(run(&config))?;

    Ok(())
}

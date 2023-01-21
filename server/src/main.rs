use ecg_hub::{app::run, config::Config, error::Error, utils::load_dotenv};
use tokio::runtime::Builder;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Error> {
    // Load variables from .env file
    let dotenv_loaded = load_dotenv()?;

    // Parse config from env
    let config: Config = envy::prefixed("HUB_").from_env()?;

    // Start logger
    tracing_subscriber::registry()
        .with(config.log_filter())
        .with(fmt::layer())
        .init();

    if dotenv_loaded {
        tracing::info!(".env file has been loaded");
    }

    // Run the server
    Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
        .block_on(run(&config))?;

    Ok(())
}

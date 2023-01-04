use std::str::FromStr;

use common::hub::{HubApiVersion, HubMode, HubStatus};
use serde::{de, Deserialize, Deserializer};
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const STATUS: HubStatus = HubStatus {
    name: "Official ECG Hub",
    hub_version: VERSION,
    api_version: HubApiVersion::V1,
    #[cfg(debug_assertions)]
    mode: HubMode::Debug,
    #[cfg(not(debug_assertions))]
    mode: HubMode::Testing,
};

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub addr: String,
    pub port: u16,
    #[serde(deserialize_with = "Config::log_level_deserialize")]
    pub log_level: LevelFilter,
}

impl Config {
    pub const DEFAULT_LOG_FILTER: &[&'static str] = &["hyper=info", "mio=info"];

    fn log_level_deserialize<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
    where
        D: Deserializer<'de>,
    {
        LevelFilter::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
    }

    pub fn log_filter(&self) -> EnvFilter {
        let mut filter = EnvFilter::default().add_directive(self.log_level.into());

        for rule in Self::DEFAULT_LOG_FILTER {
            filter = filter.add_directive(rule.parse().unwrap());
        }

        filter
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: String::from("0.0.0.0"),
            #[cfg(debug_assertions)]
            port: 3030,
            #[cfg(not(debug_assertions))]
            port: 80,
            #[cfg(debug_assertions)]
            log_level: LevelFilter::DEBUG,
            #[cfg(not(debug_assertions))]
            log_level: LevelFilter::INFO,
        }
    }
}

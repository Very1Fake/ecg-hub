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
    pub db_addr: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_pass: String,
    pub db_name: String,
    pub db_pool_min: u32,
    pub db_pool_max: u32,
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

    pub fn db_uri(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, self.db_pass, self.db_addr, self.db_port, self.db_name
        )
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
            db_addr: String::from("localhost"),
            db_port: 5432,
            db_user: String::from("root"),
            db_pass: String::from("pass"),
            db_name: String::from("ecg"),
            db_pool_min: 1,
            db_pool_max: 8,
        }
    }
}

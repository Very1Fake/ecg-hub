use common::hub::{HubApiVersion, HubMode, HubStatus};
use serde::Deserialize;

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: String::from("0.0.0.0"),
            #[cfg(debug_assertions)]
            port: 3030,
            #[cfg(not(debug_assertions))]
            port: 80,
        }
    }
}

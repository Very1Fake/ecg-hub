use common::hub::{HubApiVersion, HubMode, HubStatus};

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

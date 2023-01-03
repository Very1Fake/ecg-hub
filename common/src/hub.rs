use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct HubStatus<'a> {
    pub name: &'a str,
    pub hub_version: &'a str,
    pub api_version: HubApiVersion,
    pub mode: HubMode,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HubApiVersion {
    V1,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HubMode {
    Production = 0,
    Testing,
    Debug,
}

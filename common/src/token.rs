use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
}

impl AccessToken {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

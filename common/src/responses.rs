use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
}

impl TokenResponse {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
}

impl TokenResponse {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegistrationResponse {
    pub uuid: Uuid,
}

impl RegistrationResponse {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

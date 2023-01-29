use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::UserSession;

#[derive(Deserialize, Serialize, Debug)]
pub struct RegistrationResponse {
    pub uuid: Uuid,
}

impl RegistrationResponse {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<UserSession>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game: Option<UserSession>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<UserSession>,
}

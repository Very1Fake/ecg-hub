use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct RegistrationResponse {
    pub uuid: Uuid,
}

impl RegistrationResponse {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

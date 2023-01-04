use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserData {
    pub uuid: Uuid,
    pub username: String,
    pub status: UserStatus,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum UserStatus {
    Active = 0,
    Inactive = 1,
    Banned = 2,
}

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserData {
    pub uuid: Uuid,
    pub username: String,
    pub status: UserStatus,
}

#[derive(Deserialize_repr, Serialize_repr, Clone, Copy, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[repr(i16)]
pub enum UserStatus {
    Active = 0,
    Inactive = 1,
    Banned = 2,
}

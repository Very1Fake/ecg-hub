use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserData {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub status: UserStatus,
    pub created_at: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfo {
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

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Eq, Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[repr(i16)]
pub enum ClientType {
    #[default]
    Web = 0,
    Game = 1,
    Mobile = 2,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserSession {
    pub uuid: Uuid,
    pub ct: ClientType,
    pub expires_at: i64,
    pub updated_at: i64,
    pub created_at: i64,
}

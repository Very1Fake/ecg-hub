use chrono::{DateTime, Utc};
use common::user::{UserData, UserStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub status: UserStatus,
    pub updated: DateTime<Utc>,
    pub created: DateTime<Utc>,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        Self {
            uuid: user.uuid,
            username: user.username,
            status: user.status,
        }
    }
}

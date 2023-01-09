use common::user::{UserData, UserStatus};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgQueryResult,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    Error, FromRow,
};

use crate::{types::CiText, DB};

#[derive(FromRow, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub email: CiText,
    pub password: String,
    pub status: UserStatus,
    #[sqlx(rename = "updated_at")]
    pub updated: DateTime<Utc>,
    #[sqlx(rename = "created_at")]
    pub created: DateTime<Utc>,
}

impl User {
    pub async fn find_by_username(db: &DB, username: &str) -> Result<Option<Self>, Error> {
        sqlx::query_as(r#"SELECT * FROM "User" WHERE username = $1"#)
            .bind(username)
            .fetch_optional(db)
            .await
    }

    pub async fn find_by_uuid(db: &DB, uuid: Uuid) -> Result<Option<Self>, Error> {
        sqlx::query_as(r#"SELECT * FROM "User" WHERE uuid = $1"#)
            .bind(uuid)
            .fetch_optional(db)
            .await
    }

    pub async fn insert(&self, db: &DB) -> Result<PgQueryResult, Error> {
        sqlx::query(r#"INSERT INTO "User" (uuid, username, email, password, status) VALUES ($1, $2, $3, $4, $5)"#)
            .bind(self.uuid)
            .bind(self.username.clone())
            .bind(self.email.clone())
            .bind(self.password.clone())
            .bind(self.status)
            .execute(db)
            .await
    }
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

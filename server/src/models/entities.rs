use std::collections::HashMap;

use common::user::{ClientType, UserData, UserStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::PgQueryResult,
    types::{
        chrono::{DateTime, Utc},
        Json, Uuid,
    },
    Error, FromRow,
};

use crate::{types::CiText, DB};

////////////////////////////////////////////////////////////////////////////////////////////////////
// User
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents user account data
#[derive(FromRow, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub email: CiText,
    pub password: String,
    pub other: Json<HashMap<String, Value>>,
    pub status: UserStatus,
    #[sqlx(rename = "updated_at")]
    pub updated: DateTime<Utc>,
    #[sqlx(rename = "created_at")]
    pub created: DateTime<Utc>,
}

impl User {
    pub fn new(
        uuid: Uuid,
        username: String,
        email: String,
        password: String,
        status: UserStatus,
    ) -> Self {
        Self {
            uuid,
            username,
            email: CiText(email),
            password,
            other: Json::default(),
            status,
            updated: Utc::now(),
            created: Utc::now(),
        }
    }

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
        sqlx::query(r#"INSERT INTO "User" VALUES ($1, $2, $3, $4, $5, $6)"#)
            .bind(self.uuid)
            .bind(self.username.clone())
            .bind(self.email.clone())
            .bind(self.password.clone())
            .bind(self.other.clone())
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Session
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents user session
#[derive(FromRow, Clone, Copy, Debug)]
pub struct Session {
    pub user: Uuid,
    pub uuid: Uuid,
    pub expires: DateTime<Utc>,
}

impl Session {
    pub async fn insert(&self, db: &DB, client_type: ClientType) -> Result<Self, Error> {
        sqlx::query_as(match client_type {
            ClientType::Web => r#"INSERT INTO "WebSession" VALUES ($1, $2, $3) RETURNING *;"#,
            ClientType::Game => r#"
            INSERT INTO "GameSession" VALUES ($1, $2, $3) ON CONFLICT (sub) DO UPDATE SET uuid = excluded.uuid, expires = excluded.expires RETURNING *;"#,
            ClientType::Mobile => r#"
            INSERT INTO "MobileSession" VALUES ($1, $2, $3) ON CONFLICT (sub) DO UPDATE SET uuid = excluded.uuid, expires = excluded.expires RETURNING *;"#,
        })
            .bind(self.user)
            .bind(self.uuid)
            .bind(self.expires)
            .fetch_one(db)
            .await
    }
}

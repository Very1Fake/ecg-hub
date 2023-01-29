use std::collections::HashMap;

use common::user::{ClientType, UserData, UserInfo, UserStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::PgQueryResult,
    types::{Json, Uuid},
    Error, FromRow,
};
use time::OffsetDateTime;

use crate::{types::CiText, DB};

use super::tokens::{RefreshToken, SecurityToken};

pub enum FindBy {
    Uuid,
    Sub,
    Token,
}

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
    pub updated: OffsetDateTime,
    #[sqlx(rename = "created_at")]
    pub created: OffsetDateTime,
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
            updated: OffsetDateTime::now_utc(),
            created: OffsetDateTime::now_utc(),
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

    pub async fn update_password(&self, db: &DB) -> Result<PgQueryResult, Error> {
        sqlx::query(r#"UPDATE "User" SET password = $1 WHERE uuid = $2"#)
            .bind(self.password.clone())
            .bind(self.uuid)
            .execute(db)
            .await
    }
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        Self {
            uuid: user.uuid,
            username: user.username,
            email: user.email.0,
            status: user.status,
            created_at: user.created.unix_timestamp(),
        }
    }
}

impl From<User> for UserInfo {
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
    /// Session UUID
    pub uuid: Uuid,
    /// User UUID
    pub sub: Uuid,
    /// Refresh Token UUID
    pub token: Uuid,
    /// Expire timestamp
    pub exp: OffsetDateTime,
    /// Session refresh token last rotation timestamp
    pub updated_at: OffsetDateTime,
    /// Session creation timestamp
    pub created_at: OffsetDateTime,
}

impl Session {
    fn query_new(client_type: ClientType) -> String {
        [
            "INSERT INTO \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            "\" (sub, exp) VALUES ($1, $2) ON CONFLICT (sub) DO UPDATE SET uuid = excluded.uuid, ",
            "token = excluded.token, exp = excluded.exp, created_at = default RETURNING *;",
        ]
        .concat()
    }

    fn query_delete(client_type: ClientType, by: FindBy) -> String {
        [
            "DELETE FROM \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            match by {
                FindBy::Uuid => "\" WHERE uuid = $1",
                FindBy::Sub => "\" WHERE sub = $1",
                FindBy::Token => "\" WHERE token = $1",
            },
        ]
        .concat()
    }

    fn query_find(client_type: ClientType, by: FindBy) -> String {
        [
            "SELECT * FROM \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            match by {
                FindBy::Uuid => "\" WHERE uuid = $1",
                FindBy::Sub => "\" WHERE sub = $1",
                FindBy::Token => "\" WHERE token = $1",
            },
        ]
        .concat()
    }

    fn query_refresh(client_type: ClientType) -> String {
        [
            "UPDATE \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            "\" SET token = DEFAULT, exp = $1 WHERE uuid = $2 RETURNING token;",
        ]
        .concat()
    }

    pub async fn new(db: &DB, client_type: ClientType, sub: Uuid) -> Result<Self, Error> {
        sqlx::query_as(&Self::query_new(client_type))
            .bind(sub)
            .bind(OffsetDateTime::from_unix_timestamp(RefreshToken::new_exp()).unwrap())
            .fetch_one(db)
            .await
    }

    pub async fn find_by(
        db: &DB,
        client_type: ClientType,
        uuid: Uuid,
        by: FindBy,
    ) -> Result<Option<Self>, Error> {
        sqlx::query_as(&Self::query_find(client_type, by))
            .bind(uuid)
            .fetch_optional(db)
            .await
    }

    pub async fn delete(&self, db: &DB, client_type: ClientType) -> Result<PgQueryResult, Error> {
        Self::delete_by(db, client_type, self.uuid, FindBy::Uuid).await
    }

    pub async fn delete_by(
        db: &DB,
        client_type: ClientType,
        uuid: Uuid,
        by: FindBy,
    ) -> Result<PgQueryResult, Error> {
        sqlx::query(&Self::query_delete(client_type, by))
            .bind(uuid)
            .execute(db)
            .await
    }

    pub async fn refresh(&mut self, db: &DB, client_type: ClientType) -> Result<(), Error> {
        self.exp = OffsetDateTime::from_unix_timestamp(RefreshToken::new_exp()).unwrap();
        self.token = sqlx::query_scalar(&Self::query_refresh(client_type))
            .bind(self.exp)
            .bind(self.uuid)
            .fetch_one(db)
            .await?;

        Ok(())
    }
}

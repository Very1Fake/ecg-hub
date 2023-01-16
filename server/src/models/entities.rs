use std::collections::HashMap;

use chrono::TimeZone;
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

use super::claims::{RefreshTokenClaims, SecurityToken};

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
    /// User UUID
    pub sub: Uuid,
    /// Session UUID
    pub uuid: Uuid,
    /// Expire timestamp
    pub exp: DateTime<Utc>,
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
            "\" VALUES ($1, default, $2) ON CONFLICT (sub) DO UPDATE SET uuid = excluded.uuid, exp = excluded.exp RETURNING *;"
        ].concat()
    }

    fn query_update(client_type: ClientType) -> String {
        [
            "UPDATE \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            "\" SET exp = $2 WHERE sub = $1",
        ]
        .concat()
    }

    fn query_find(client_type: ClientType) -> String {
        [
            "SELECT * FROM \"",
            match client_type {
                ClientType::Web => "WebSession",
                ClientType::Game => "GameSession",
                ClientType::Mobile => "MobileSession",
            },
            "\" WHERE sub = $1",
        ]
        .concat()
    }

    pub async fn new(
        db: &DB,
        client_type: ClientType,
        user: Uuid,
        exp: u64,
    ) -> Result<Self, Error> {
        sqlx::query_as(&Self::query_new(client_type))
            .bind(user)
            .bind(Utc.timestamp_opt(exp as i64, 0).unwrap())
            .fetch_one(db)
            .await
    }

    pub async fn update(&self, db: &DB, client_type: ClientType) -> Result<Self, Error> {
        sqlx::query_as(&Self::query_update(client_type))
            .bind(self.sub)
            .bind(self.exp)
            .fetch_one(db)
            .await
    }

    pub async fn find_by_sub(
        db: &DB,
        client_type: ClientType,
        sub: Uuid,
    ) -> Result<Option<Self>, Error> {
        sqlx::query_as(&Self::query_find(client_type))
            .bind(sub)
            .fetch_optional(db)
            .await
    }

    pub async fn refresh(
        db: &DB,
        client_type: ClientType,
        user: Uuid,
        new: bool,
    ) -> Result<(Self, Uuid), Error> {
        let with_access_uuid = |session| (session, Uuid::new_v4());

        let exp = RefreshTokenClaims::new_exp();

        if !new {
            if let Some(mut session) = Self::find_by_sub(db, client_type, user).await? {
                session.exp = Utc.timestamp_opt(exp as i64, 0).unwrap();
                return session.update(db, client_type).await.map(with_access_uuid);
            }
        }

        Self::new(db, client_type, user, exp)
            .await
            .map(with_access_uuid)
    }
}

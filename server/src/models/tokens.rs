use axum::{
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    TypedHeader,
};
use axum_extra::extract::cookie::Cookie;
use common::user::ClientType;
use hyper::StatusCode;
use jsonwebtoken::{decode, encode, get_current_timestamp, Algorithm, Header};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;

use crate::keys::Keys;

use super::entities::Session;

pub trait SecurityToken: DeserializeOwned + Serialize
where
    Self: Sized,
{
    const LIFETIME: i64;

    fn new_nbf() -> i64 {
        get_current_timestamp() as i64
    }

    fn new_exp() -> i64 {
        get_current_timestamp() as i64 + Self::LIFETIME
    }

    fn sign(&self, keys: &Keys) -> String {
        encode(&Header::new(Algorithm::EdDSA), self, &keys.encoding).expect("Failed to sign token")
    }

    fn decode(token: &str, keys: &Keys) -> Result<Self, jsonwebtoken::errors::Error> {
        decode(token, &keys.decoding, &keys.validation).map(|data| data.claims)
    }
}

/// Contains refresh token claims
#[derive(Deserialize, Serialize, Debug)]
pub struct RefreshToken {
    /// Session UUID
    pub sess: Uuid,
    /// User UUID
    pub sub: Uuid,
    /// Refresh Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Not before time (UTC timestamp)
    pub nbf: i64,
    /// Client Type
    pub ct: ClientType,
}

impl RefreshToken {
    pub const COOKIE_NAME: &str = "hub-rt";
    pub const ROTATION_PERIOD: Duration = Duration::days(7);

    #[inline]
    pub const fn new_raw(
        sess: Uuid,
        sub: Uuid,
        jti: Uuid,
        exp: i64,
        nbf: i64,
        ct: ClientType,
    ) -> Self {
        Self {
            sess,
            sub,
            jti,
            exp,
            nbf,
            ct,
        }
    }

    pub fn new(sess: Uuid, sub: Uuid, jti: Uuid, ct: ClientType) -> Self {
        Self::new_raw(sess, sub, jti, Self::new_exp(), Self::new_nbf(), ct)
    }

    pub fn to_cookie(&self, keys: &Keys) -> Cookie<'static> {
        let mut cookie = Cookie::new("hub-rt", self.sign(keys));
        cookie.set_max_age(Some(Duration::seconds(RefreshToken::LIFETIME)));
        cookie.set_http_only(true);
        cookie
    }
}

impl SecurityToken for RefreshToken {
    /// Refresh token lifetime: 6 month
    const LIFETIME: i64 = 60 * 60 * 24 * 30 * 6;
}

impl From<(&Session, ClientType)> for RefreshToken {
    fn from((session, ct): (&Session, ClientType)) -> Self {
        Self::new_raw(
            session.uuid,
            session.sub,
            session.token,
            session.exp.unix_timestamp(),
            Self::new_nbf(),
            ct,
        )
    }
}

/// Contains access token claims
#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
    /// Session UUID
    pub iss: Uuid,
    /// User UUID
    pub sub: Uuid,
    /// Access Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Client Type
    pub ct: ClientType,
}

impl AccessToken {
    pub fn new(iss: Uuid, sub: Uuid, ct: ClientType) -> Self {
        Self {
            iss,
            sub,
            jti: Uuid::new_v4(),
            exp: Self::new_exp(),
            ct,
        }
    }
}

impl SecurityToken for AccessToken {
    /// Access token lifetime: 1 minute
    const LIFETIME: i64 = 60;
}

impl From<(&Session, ClientType)> for AccessToken {
    fn from((session, ct): (&Session, ClientType)) -> Self {
        Self::new(session.uuid, session.sub, ct)
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AccessToken
where
    Keys: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::EXPECTATION_FAILED)?;

        let keys = Keys::from_ref(state);

        Self::decode(bearer.token(), &keys).map_err(|_| StatusCode::FORBIDDEN)
    }
}

/// Contains Player Identity Token (PIT) claims
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerIdentityToken {
    /// Server ID (SID)
    pub aud: String,
    /// User UUID
    pub sub: Uuid,
    /// Access Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Not before time (UTC timestamp)
    pub nbf: i64,
    /// Client Type
    pub ct: ClientType,
}

impl PlayerIdentityToken {
    #[inline]
    pub const fn new_raw(
        aud: String,
        sub: Uuid,
        jti: Uuid,
        exp: i64,
        nbf: i64,
        ct: ClientType,
    ) -> Self {
        Self {
            aud,
            sub,
            jti,
            exp,
            nbf,
            ct,
        }
    }

    pub fn new(aud: String, sub: Uuid, ct: ClientType) -> Self {
        Self::new_raw(
            aud,
            sub,
            Uuid::new_v4(),
            Self::new_exp(),
            Self::new_nbf(),
            ct,
        )
    }
}

impl SecurityToken for PlayerIdentityToken {
    /// PIT lifetime: 15 seconds
    const LIFETIME: i64 = 15;
}

use axum::{
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    TypedHeader,
};
use axum_extra::extract::cookie::Cookie;
use common::user::ClientType;
use hyper::StatusCode;
use jsonwebtoken::{decode, encode, get_current_timestamp, Algorithm, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;

use crate::keys::Keys;

pub trait SecurityToken: DeserializeOwned + Serialize
where
    Self: Sized,
{
    const LIFETIME: i64;

    fn new_exp() -> i64 {
        get_current_timestamp() as i64 + Self::LIFETIME
    }

    fn sign(&self, keys: &Keys) -> String {
        encode(&Header::new(Algorithm::EdDSA), self, &keys.encoding).expect("Failed to sign token")
    }

    fn decode(token: &str, keys: &Keys) -> Result<Self, jsonwebtoken::errors::Error> {
        decode(token, &keys.decoding, &Validation::new(Algorithm::EdDSA)).map(|data| data.claims)
    }
}

/// Contains refresh token claims
#[derive(Deserialize, Serialize, Debug)]
pub struct RefreshToken {
    /// User UUID
    pub sub: Uuid,
    /// Refresh Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Client Type
    pub ct: ClientType,
}

impl RefreshToken {
    pub const COOKIE_NAME: &str = "hub-rt";

    pub fn new(sub: Uuid, jti: Uuid, ct: ClientType) -> Self {
        Self {
            ct,
            sub,
            jti,
            exp: Self::new_exp(),
        }
    }

    pub fn to_cookie(&self, keys: &Keys) -> Cookie<'static> {
        let mut cookie = Cookie::new("hub-rt", self.sign(keys));
        cookie.set_max_age(Some(Duration::seconds(RefreshToken::LIFETIME)));
        cookie.set_http_only(true);
        cookie
    }
}

impl SecurityToken for RefreshToken {
    /// Tokens lifespan: 1 month
    const LIFETIME: i64 = 60 * 60 * 24 * 30;
}

// TODO: Store refresh token id
/// Contains access token claims
#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
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
    pub fn new(sub: Uuid, jti: Uuid, ct: ClientType) -> Self {
        Self {
            ct,
            sub,
            jti,
            exp: Self::new_exp(),
        }
    }
}

impl SecurityToken for AccessToken {
    /// Tokens lifespan: 5 minutes
    const LIFETIME: i64 = 60 * 5;
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

        Self::decode(bearer.token(), &keys).map_err(|_| StatusCode::BAD_REQUEST)
    }
}

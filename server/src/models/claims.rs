use common::user::ClientType;
use jsonwebtoken::get_current_timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait SecurityToken {
    const LIFESPAN: u64;

    fn new_exp() -> u64 {
        get_current_timestamp() + Self::LIFESPAN
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RefreshTokenClaims {
    /// Client Type
    pub aud: ClientType,
    /// User UUID
    pub sub: Uuid,
    /// Refresh Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: u64,
}

impl RefreshTokenClaims {
    pub fn new(aud: ClientType, sub: Uuid, jti: Uuid, exp: u64) -> Self {
        Self { aud, sub, jti, exp }
    }
}

impl SecurityToken for RefreshTokenClaims {
    /// Tokens lifespan: 1 month
    const LIFESPAN: u64 = 60 * 60 * 24 * 30;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessTokenClaims {
    /// Client Type
    pub aud: ClientType,
    /// User UUID
    pub sub: Uuid,
    /// Access Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: u64,
}

impl AccessTokenClaims {
    pub fn new(aud: ClientType, sub: Uuid, jti: Uuid, exp: u64) -> Self {
        Self { aud, sub, jti, exp }
    }
}

impl SecurityToken for AccessTokenClaims {
    /// Tokens lifespan: 5 minutes
    const LIFESPAN: u64 = 60 * 5;
}

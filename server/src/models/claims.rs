use common::user::ClientType;
use jsonwebtoken::get_current_timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait SecurityToken {
    const LIFETIME: i64;

    fn new_exp() -> i64 {
        get_current_timestamp() as i64 + Self::LIFETIME
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RefreshTokenClaims {
    /// User UUID
    pub sub: Uuid,
    /// Refresh Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Client Type
    pub ct: ClientType,
}

impl RefreshTokenClaims {
    pub fn new(sub: Uuid, jti: Uuid, ct: ClientType) -> Self {
        Self {
            ct,
            sub,
            jti,
            exp: Self::new_exp(),
        }
    }
}

impl SecurityToken for RefreshTokenClaims {
    /// Tokens lifespan: 1 month
    const LIFETIME: i64 = 60 * 60 * 24 * 30;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessTokenClaims {
    /// User UUID
    pub sub: Uuid,
    /// Access Token UUID
    pub jti: Uuid,
    /// Expire time (UTC timestamp)
    pub exp: i64,
    /// Client Type
    pub ct: ClientType,
}

impl AccessTokenClaims {
    pub fn new(sub: Uuid, jti: Uuid, ct: ClientType) -> Self {
        Self {
            ct,
            sub,
            jti,
            exp: Self::new_exp(),
        }
    }
}

impl SecurityToken for AccessTokenClaims {
    /// Tokens lifespan: 5 minutes
    const LIFETIME: i64 = 60 * 5;
}

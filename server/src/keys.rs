use ed25519_compact::{KeyPair, Seed};
use hex::ToHex;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};

use crate::models::claims::{AccessTokenClaims, RefreshTokenClaims};

pub struct Keys {
    pub pair: KeyPair,

    // JWT
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,

    // Cached values
    pub public_hex: String,
    pub public_pem: String,
}

impl Keys {
    pub fn new(pair: KeyPair) -> Self {
        let public_pem = pair.pk.to_pem();

        Self {
            encoding: EncodingKey::from_ed_pem(pair.sk.to_pem().as_bytes()).unwrap(),
            decoding: DecodingKey::from_ed_pem(public_pem.as_bytes()).unwrap(),
            public_hex: pair.pk.as_slice().encode_hex(),
            pair,
            public_pem,
        }
    }

    pub fn rand() -> Self {
        Self::new(KeyPair::generate())
    }

    pub fn sign_refresh_token(&self, claims: &RefreshTokenClaims) -> String {
        encode(&Header::new(Algorithm::EdDSA), claims, &self.encoding)
            .expect("Failed to generate refresh token")
    }

    pub fn sign_access_token(&self, claims: &AccessTokenClaims) -> String {
        encode(&Header::new(Algorithm::EdDSA), claims, &self.encoding)
            .expect("Failed to generate access token")
    }
}

impl From<[u8; 32]> for Keys {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(KeyPair::from_seed(Seed::new(bytes)))
    }
}

use std::sync::Arc;

use axum::extract::FromRef;
use ed25519_compact::{KeyPair, Seed};
use hex::ToHex;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};

use crate::app::HubState;

#[derive(Clone)]
pub struct Keys {
    pub pair: KeyPair,

    // JWT
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,

    // Cached values
    pub public_hex: String,
    pub public_pem: String,

    pub validation: Validation,
}

impl Keys {
    pub fn new(pair: KeyPair) -> Self {
        let public_pem = pair.pk.to_pem();

        let mut validation = Validation::new(Algorithm::EdDSA);
        // Allowed time error: 1 second
        validation.leeway = 1;

        Self {
            encoding: EncodingKey::from_ed_pem(pair.sk.to_pem().as_bytes()).unwrap(),
            decoding: DecodingKey::from_ed_pem(public_pem.as_bytes()).unwrap(),
            public_hex: pair.pk.as_slice().encode_hex(),
            pair,
            public_pem,
            validation,
        }
    }

    pub fn rand() -> Self {
        Self::new(KeyPair::generate())
    }
}

impl From<[u8; 32]> for Keys {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(KeyPair::from_seed(Seed::new(bytes)))
    }
}

impl FromRef<Arc<HubState>> for Keys {
    fn from_ref(state: &Arc<HubState>) -> Self {
        state.keys.clone()
    }
}

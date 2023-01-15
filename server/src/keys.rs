use ed25519_dalek::{PublicKey, SecretKey};
use hex::ToHex;
use rand_7::rngs::OsRng;

#[derive(Debug)]
pub struct Keys {
    pub private: SecretKey,
    pub public: PublicKey,
    pub public_hex: String,
}

impl Keys {
    pub fn new(private: SecretKey) -> Self {
        let public = PublicKey::from(&private);

        Self {
            private,
            public,
            public_hex: public.as_bytes().encode_hex(),
        }
    }

    pub fn rand() -> Self {
        Self::new(SecretKey::generate(&mut OsRng {}))
    }
}

impl From<&[u8; 32]> for Keys {
    fn from(bytes: &[u8; 32]) -> Self {
        Self::new(SecretKey::from_bytes(bytes).expect("Failed to parse private key from bytes"))
    }
}

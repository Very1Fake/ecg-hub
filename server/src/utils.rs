use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;

use crate::error::Error;

pub fn load_dotenv() -> Result<bool, Error> {
    Ok(match dotenvy::dotenv() {
        Ok(_) => true,
        Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => false,
        Err(err) => return Err(err.into()),
    })
}

pub fn hash_password(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .expect("Failed to generate password hash")
        .to_string()
}

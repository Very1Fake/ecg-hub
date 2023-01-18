use common::user::ClientType;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
    pub static ref USERNAME_REGEX: Regex = Regex::new("^[a-zA-Z0-9_]{3,24}$").unwrap();
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum KeyFormat {
    #[default]
    Hex,
    Pem,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct KeyFormatQuery {
    pub format: KeyFormat,
}

#[derive(Deserialize, Debug)]
pub struct UserIdQuery {
    pub uuid: Option<Uuid>,
    pub username: Option<String>,
}

#[derive(Validate, Deserialize, Debug)]
pub struct RegisterForm {
    #[validate(regex = "USERNAME_REGEX")]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
}

#[derive(Validate, Deserialize, Debug)]
pub struct LoginForm {
    #[validate(regex = "USERNAME_REGEX")]
    pub username: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
    #[serde(default)]
    pub ct: ClientType,
}

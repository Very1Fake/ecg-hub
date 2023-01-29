use common::user::ClientType;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
    /// Regular expression for username
    pub static ref USERNAME_REGEX: Regex = Regex::new("^[a-zA-Z0-9_]{3,24}$").unwrap();
    /// Regular expression for Server ID (e.g. "eYp1Zl1td14E")
    pub static ref SID_REGEX: Regex = Regex::new("^[a-zA-Z0-9]{12}$").unwrap();
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

#[derive(Validate, Deserialize, Debug)]
pub struct UserInfoQuery {
    pub uuid: Option<Uuid>,
    #[validate(regex = "USERNAME_REGEX")]
    pub username: Option<String>,
}

#[derive(Validate, Deserialize, Debug)]
pub struct PITQuery {
    #[validate(regex = "SID_REGEX")]
    pub sid: String,
}

#[derive(Validate, Deserialize, Debug)]
pub struct RegisterBody {
    #[validate(regex = "USERNAME_REGEX")]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
}

#[derive(Validate, Deserialize, Debug)]
pub struct LoginBody {
    #[validate(regex = "USERNAME_REGEX")]
    pub username: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
    #[serde(default)]
    pub ct: ClientType,
}

#[derive(Validate, Deserialize, Debug)]
pub struct PasswordChangeBody {
    #[validate(length(min = 6, max = 64))]
    pub old_password: String,
    #[validate(length(min = 6, max = 64))]
    pub new_password: String,
}

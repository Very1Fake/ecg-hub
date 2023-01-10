use common::user::ClientType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug)]
pub struct UserIdQuery {
    pub uuid: Option<Uuid>,
    pub username: Option<String>,
}

#[derive(Validate, Deserialize, Serialize, Debug)]
pub struct RegisterForm {
    #[validate(length(min = 3, max = 24))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
}

#[derive(Validate, Deserialize, Serialize, Debug)]
pub struct LoginForm {
    #[validate(length(min = 3, max = 24))]
    pub username: String,
    #[validate(length(min = 6, max = 64))]
    pub password: String,
    #[serde(default)]
    pub ct: ClientType,
}

use std::ops::Deref;

use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Type, Deserialize, Serialize, Clone, Debug)]
#[sqlx(type_name = "citext")]
#[serde(transparent)]
pub struct CiText(pub String);

impl Deref for CiText {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for CiText {
    fn from(string: String) -> Self {
        Self(string)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenPair {
    pub refresh: String,
    pub access: String,
}

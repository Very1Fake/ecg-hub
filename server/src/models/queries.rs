use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct UserIdQuery {
    pub uuid: Option<Uuid>,
    pub username: Option<String>,
}

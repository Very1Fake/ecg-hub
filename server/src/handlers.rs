use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use hyper::StatusCode;

use common::{hub::HubStatus, user::UserData};

use crate::{app::HubState, config::STATUS, models::queries::UserIdQuery};

pub async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

pub async fn user_get(
    State(state): State<Arc<HubState>>,
    user_id: Query<UserIdQuery>,
) -> Result<Json<UserData>, StatusCode> {
    Ok(Json(
        match (user_id.uuid, &user_id.username) {
            (Some(uuid), _) => {
                if let Some(user) = state.users.get(&uuid) {
                    user.clone()
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            (None, Some(username)) => {
                if let Some(user) = state
                    .users
                    .values()
                    .find(|&user| user.username == *username)
                {
                    user.clone()
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        }
        .into(),
    ))
}

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use hyper::StatusCode;

use common::{
    hub::HubStatus,
    user::{UserData, UserStatus},
};
use sqlx::types::Uuid;
use validator::Validate;

use crate::{
    app::HubState,
    config::STATUS,
    error::Error,
    models::{
        entities::User,
        parsers::{RegisterForm, UserIdQuery},
    },
};

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
                if let Some(user) = User::find_by_uuid(&state.db, uuid).await.unwrap() {
                    user
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            (None, Some(username)) => {
                if let Some(user) = User::find_by_username(&state.db, username).await.unwrap() {
                    user
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        }
        .into(),
    ))
}

pub async fn user_post(
    State(state): State<Arc<HubState>>,
    Json(form): Json<RegisterForm>,
) -> Result<StatusCode, Error> {
    if form.validate().is_ok() {
        User {
            uuid: Uuid::new_v4(),
            username: form.username,
            email: form.email.into(),
            password: form.password,
            status: UserStatus::Active,
            updated: Utc::now(),
            created: Utc::now(),
        }
        .insert(&state.db)
        .await?;

        Ok(StatusCode::CREATED)
    } else {
        Ok(StatusCode::BAD_REQUEST)
    }
}

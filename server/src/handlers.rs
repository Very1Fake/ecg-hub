use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use hyper::StatusCode;

use common::{
    hub::HubStatus,
    user::{UserData, UserStatus},
};
use rand_core::OsRng;
use serde_json::json;
use sqlx::{postgres::PgDatabaseError, types::Uuid};
use tracing::error;
use validator::Validate;

use crate::{
    app::HubState,
    config::STATUS,
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
) -> impl IntoResponse {
    if form.validate().is_ok() {
        let RegisterForm {
            username,
            email,
            password,
        } = form;

        match User::new(
            Uuid::new_v4(),
            username.clone(),
            email.clone(),
            Argon2::default()
                .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
                .expect("password hashing failed")
                .to_string(),
            UserStatus::Active,
        )
        .insert(&state.db)
        .await
        {
            Err(sqlx::Error::Database(err))
                if err.try_downcast_ref::<PgDatabaseError>().is_some() =>
            {
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "err": true,
                        "msg": match err.downcast::<PgDatabaseError>()
                            .constraint()
                            .expect("db error without constraint")
                        {
                            "User_username_key" => format!("username '{username}' already taken"),
                            "User_email_key" => format!("email '{email}' already taken"),
                            _ => String::from("db error"),
                        }
                    })),
                )
                    .into_response()
            }
            Err(err) => {
                error!(?err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => StatusCode::CREATED.into_response(),
        }
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn auth_login() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn token_refresh() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn token_revoke() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn token_revoke_all() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Form, Json,
};
use hyper::StatusCode;
use rand_8::rngs::OsRng;
use sqlx::{postgres::PgDatabaseError, types::Uuid};
use tracing::error;
use validator::Validate;

use common::{
    hub::HubStatus,
    user::{UserData, UserStatus},
};

use crate::{
    app::HubState,
    config::STATUS,
    models::{
        entities::User,
        parsers::{LoginForm, RegisterForm, UserIdQuery},
    },
};

pub async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

pub async fn pubkey(State(state): State<Arc<HubState>>) -> String {
    state.keys.public_hex.clone()
}

// User

pub async fn user_get(
    State(state): State<Arc<HubState>>,
    user_id: Query<UserIdQuery>,
) -> Result<Json<UserData>, StatusCode> {
    Ok(Json(
        match (user_id.uuid, &user_id.username) {
            (Some(uuid), _) => {
                if let Some(user) = User::find_by_uuid(&state.db, uuid)
                    .await
                    .expect("failed to retrieve user data from db")
                {
                    user
                } else {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            (None, Some(username)) => {
                if let Some(user) = User::find_by_username(&state.db, username)
                    .await
                    .expect("failed to retrieve user data from db")
                {
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
                    match err
                        .downcast::<PgDatabaseError>()
                        .constraint()
                        .expect("db error without constraint")
                    {
                        "User_username_key" => format!("username '{username}' already taken"),
                        "User_email_key" => format!("email '{email}' already taken"),
                        _ => String::from("db error"),
                    },
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

// Security

pub async fn auth_login(
    State(state): State<Arc<HubState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    if form.validate().is_ok() {
        let LoginForm {
            username, password, ..
        } = form;

        if let Some(user) = User::find_by_username(&state.db, &username)
            .await
            .expect("failed to retrieve user data from db")
        {
            if Argon2::default()
                .verify_password(
                    password.as_bytes(),
                    &PasswordHash::new(&user.password).expect("failed to parse password hash"),
                )
                .is_ok()
            {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::BAD_REQUEST
    }
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

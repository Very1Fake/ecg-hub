use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Form, Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use hyper::StatusCode;
use rand::rngs::OsRng;
use sqlx::{postgres::PgDatabaseError, types::Uuid};
use time::Duration;
use tracing::error;
use validator::Validate;

use common::{
    hub::HubStatus,
    token::AccessToken,
    user::{UserData, UserStatus},
};

use crate::{
    app::HubState,
    config::STATUS,
    models::{
        claims::{AccessTokenClaims, RefreshTokenClaims, SecurityToken},
        entities::{Session, User},
        parsers::{KeyFormat, KeyFormatQuery, LoginForm, RegisterForm, UserIdQuery},
    },
};

pub async fn info() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

pub async fn pubkey(
    State(state): State<Arc<HubState>>,
    Query(format): Query<KeyFormatQuery>,
) -> String {
    match format.format {
        KeyFormat::Hex => state.keys.public_hex.clone(),
        KeyFormat::Pem => state.keys.public_pem.clone(),
    }
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
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Json<AccessToken>), StatusCode> {
    if form.validate().is_ok() {
        let LoginForm {
            username,
            password,
            ct,
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
                return match user.status {
                    UserStatus::Active => {
                        let (session, access_uuid) =
                            Session::refresh(&state.db, ct, user.uuid, true)
                                .await
                                .unwrap();

                        Ok((
                            jar.add(refresh_token_cookie(state.keys.sign_refresh_token(
                                &RefreshTokenClaims::new(user.uuid, session.uuid, ct),
                            ))),
                            Json(AccessToken::new(state.keys.sign_access_token(
                                &AccessTokenClaims::new(user.uuid, access_uuid, ct),
                            ))),
                        ))
                    }
                    UserStatus::Inactive => Err(StatusCode::IM_A_TEAPOT),
                    UserStatus::Banned => Err(StatusCode::GONE),
                };
            }
        }
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// TODO: Add refresh token auto rotation
pub async fn token_refresh() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

// TODO: Add authentication middleware
pub async fn token_revoke() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn token_revoke_all() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

// Utils

fn refresh_token_cookie(token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new("hub-rt", token);
    cookie.set_max_age(Duration::seconds(RefreshTokenClaims::LIFETIME));
    cookie.set_http_only(true);
    cookie
}

use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use hyper::StatusCode;
use rand::rngs::OsRng;
use sqlx::{postgres::PgDatabaseError, types::Uuid};
use tracing::error;
use validator::Validate;

use common::{
    hub::HubStatus,
    responses::{RegistrationResponse, TokenResponse},
    user::{ClientType, UserData, UserInfo, UserStatus},
};

use crate::{
    app::HubState,
    config::STATUS,
    models::{
        entities::{Session, User},
        parsers::{KeyFormat, KeyFormatQuery, LoginBody, RegisterBody, UserIdQuery},
        tokens::{AccessToken, RefreshToken, SecurityToken},
    },
};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

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

pub async fn user_info(
    State(state): State<Arc<HubState>>,
    user_id: Query<UserIdQuery>,
) -> Result<Json<UserInfo>, StatusCode> {
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

pub async fn user_data(
    State(state): State<Arc<HubState>>,
    AccessToken { sub, .. }: AccessToken,
) -> Result<Json<UserData>, StatusCode> {
    Ok(Json(
        if let Some(user) = User::find_by_uuid(&state.db, sub)
            .await
            .expect("failed to retrieve user info from db")
        {
            user.into()
        } else {
            return Err(StatusCode::NOT_FOUND);
        },
    ))
}

// Security

pub async fn user_login(
    State(state): State<Arc<HubState>>,
    jar: CookieJar,
    Json(body): Json<LoginBody>,
) -> Result<(CookieJar, Json<TokenResponse>), StatusCode> {
    if body.validate().is_ok() {
        let LoginBody {
            username,
            password,
            ct,
        } = body;

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
                            jar.add(
                                RefreshToken::new(user.uuid, session.uuid, ct)
                                    .to_cookie(&state.keys),
                            ),
                            Json(TokenResponse::new(
                                AccessToken::new(session.uuid, user.uuid, access_uuid, ct)
                                    .sign(&state.keys),
                            )),
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

pub async fn user_register(
    State(state): State<Arc<HubState>>,
    Json(body): Json<RegisterBody>,
) -> impl IntoResponse {
    if body.validate().is_ok() {
        let RegisterBody {
            username,
            email,
            password,
        } = body;

        let uuid = Uuid::new_v4();

        match User::new(
            uuid,
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
            Ok(_) => (StatusCode::CREATED, Json(RegistrationResponse::new(uuid))).into_response(),
        }
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

// TODO: Add refresh token auto rotation
pub async fn token_refresh(
    State(state): State<Arc<HubState>>,
    jar: CookieJar,
) -> Result<Json<TokenResponse>, StatusCode> {
    if let Some(cookie) = jar.get("hub-rt") {
        if let Ok(token) = RefreshToken::decode(cookie.value(), &state.keys) {
            // TODO: Use better logging system
            if let Some(session) = Session::find_by_uuid(&state.db, token.ct, token.jti)
                .await
                .expect("Failed to execute query while searching for session (token/refresh)")
            {
                Ok(Json(TokenResponse::new(
                    AccessToken::new(session.uuid, session.sub, Uuid::new_v4(), token.ct)
                        .sign(&state.keys),
                )))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        Err(StatusCode::EXPECTATION_FAILED)
    }
}

pub async fn token_revoke(
    State(state): State<Arc<HubState>>,
    AccessToken { iss, ct, .. }: AccessToken,
) -> StatusCode {
    if let Some(session) = Session::find_by_uuid(&state.db, ct, iss)
        .await
        .expect("Failed to execute query while searching for session (token/revoke)")
    {
        match session.delete(&state.db, ct).await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

// TODO: Renew current token
pub async fn token_revoke_all(
    State(state): State<Arc<HubState>>,
    AccessToken { iss, sub, ct, .. }: AccessToken,
) -> StatusCode {
    if Session::find_by_uuid(&state.db, ct, iss)
        .await
        .expect("Failed to execute query while searching for session (token/revoke_all)")
        .is_some()
    {
        macro_rules! delete_session {
            ($ct: expr) => {
                if ct != $ct {
                    if Session::delete_by_sub(&state.db, sub, $ct).await.is_err() {
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }
                }
            };
        }

        delete_session!(ClientType::Web);
        delete_session!(ClientType::Game);
        delete_session!(ClientType::Mobile);

        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

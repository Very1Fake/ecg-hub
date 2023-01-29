use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use hyper::StatusCode;
use sqlx::{postgres::PgDatabaseError, types::Uuid};
use time::OffsetDateTime;
use tracing::error;
use validator::Validate;

use common::{
    hub::HubStatus,
    responses::RegistrationResponse,
    user::{ClientType, UserData, UserInfo, UserStatus},
};

use crate::{
    app::HubState,
    config::STATUS,
    models::{
        entities::{Session, User},
        parsers::{
            KeyFormat, KeyFormatQuery, LoginBody, PITQuery, PasswordChangeBody, RegisterBody,
            UserInfoQuery,
        },
        tokens::{AccessToken, PlayerIdentityToken, RefreshToken, SecurityToken},
    },
    utils::hash_password,
};

/// Public Endpoint: For health checks
pub async fn health() -> StatusCode {
    StatusCode::OK
}

/// Public Endpoint: Returns hub status
pub async fn status() -> Json<HubStatus<'static>> {
    Json(STATUS)
}

/// Public Endpoint: Returns the public key used to verify the signature of the tokens
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

/// Public Endpoint: Looks up for the uuid and username of the account
pub async fn user_info(
    State(state): State<Arc<HubState>>,
    user_id: Query<UserInfoQuery>,
) -> Result<Json<UserInfo>, StatusCode> {
    if user_id.validate().is_ok() {
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
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// Private Endpoint: Allows the user to retrieve their personal data
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

/// Private Endpoint: Allows user to create a new session and a refresh/access token pair
pub async fn user_login(
    State(state): State<Arc<HubState>>,
    jar: CookieJar,
    Json(body): Json<LoginBody>,
) -> Result<(CookieJar, String), StatusCode> {
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
                        let session = Session::new(&state.db, ct, user.uuid).await.unwrap();

                        Ok((
                            jar.add(
                                RefreshToken::new(user.uuid, session.uuid, ct)
                                    .to_cookie(&state.keys),
                            ),
                            AccessToken::new(session.uuid, user.uuid, ct).sign(&state.keys),
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

/// Endpoint: Creates new a user account
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
            hash_password(&password),
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

/// Private Endpoint: Allows user to change the password with their old password and access token
pub async fn user_password(
    State(state): State<Arc<HubState>>,
    AccessToken { sub, .. }: AccessToken,
    Json(body): Json<PasswordChangeBody>,
) -> StatusCode {
    if body.validate().is_ok() {
        if let Some(mut user) = User::find_by_uuid(&state.db, sub)
            .await
            .expect("Failed to find user")
        {
            if Argon2::default()
                .verify_password(
                    body.old_password.as_bytes(),
                    &PasswordHash::new(&user.password).expect("Failed to parse password hash"),
                )
                .is_ok()
            {
                user.password = hash_password(&body.new_password);
                user.update_password(&state.db)
                    .await
                    .expect("Failed to make update request to DB");
                StatusCode::OK
            } else {
                StatusCode::NOT_MODIFIED
            }
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

/// Private Endpoint: Generates a new access token using the refresh token
pub async fn token_refresh(
    State(state): State<Arc<HubState>>,
    mut jar: CookieJar,
) -> Result<(CookieJar, String), StatusCode> {
    if let Some(cookie) = jar.get("hub-rt") {
        if let Ok(RefreshToken { sub, jti, ct, .. }) =
            RefreshToken::decode(cookie.value(), &state.keys)
        {
            // TODO: Use better logging system
            if let Some(mut session) = Session::find_by_uuid(&state.db, ct, jti)
                .await
                .expect("Failed to execute query while searching for session (token/refresh)")
            {
                if session.exp - OffsetDateTime::now_utc() < RefreshToken::ROTATION_PERIOD {
                    session = Session::new(&state.db, ct, sub)
                        .await
                        .expect("Failed to refresh session");
                    jar = jar.add(RefreshToken::from((&session, ct)).to_cookie(&state.keys));
                }

                Ok((
                    jar,
                    AccessToken::new(session.uuid, session.sub, ct).sign(&state.keys),
                ))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        Err(StatusCode::EXPECTATION_FAILED)
    }
}

/// Private Endpoint: Ends current session with the access token
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

/// Private Endpoint: Ends all user session and creates a new one for current client type
pub async fn token_revoke_all(
    State(state): State<Arc<HubState>>,
    jar: CookieJar,
    AccessToken { iss, sub, ct, .. }: AccessToken,
) -> Result<CookieJar, StatusCode> {
    if Session::find_by_uuid(&state.db, ct, iss)
        .await
        .expect("Failed to execute query while searching for session (token/revoke_all)")
        .is_some()
    {
        macro_rules! delete_session {
            ($ct: expr) => {
                if Session::delete_by_sub(&state.db, sub, $ct).await.is_err() {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
        }

        delete_session!(ClientType::Web);
        delete_session!(ClientType::Game);
        delete_session!(ClientType::Mobile);

        Ok(jar.add(
            RefreshToken::from((
                &Session::new(&state.db, ct, sub)
                    .await
                    .expect("Failed to rotate refresh token"),
                ct,
            ))
            .to_cookie(&state.keys),
        ))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Private Endpoint: Allows user to generate PIT for joining game servers
pub async fn token_pit(
    State(state): State<Arc<HubState>>,
    AccessToken { sub, ct, .. }: AccessToken,
    Query(query): Query<PITQuery>,
) -> Result<String, StatusCode> {
    if query.validate().is_ok() {
        Ok(PlayerIdentityToken::new(query.sid, sub, ct).sign(&state.keys))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

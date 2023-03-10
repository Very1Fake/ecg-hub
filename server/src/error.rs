use std::{io, net::AddrParseError};

use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

macro_rules! impl_from_error {
    ($from: ty, $to: expr) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                $to(err)
            }
        }
    };
}

// TODO: Make external error type
#[derive(Debug)]
pub enum Error {
    ConfigError(String),
    JWTError(jsonwebtoken::errors::Error),
    SqlxError(sqlx::Error),
    HyperError(hyper::Error),
    EnvyError(envy::Error),
    DotEnvError(dotenvy::Error),
    IOError(io::Error),
    AddrParseError(AddrParseError),
}

impl_from_error!(jsonwebtoken::errors::Error, Error::JWTError);
impl_from_error!(sqlx::Error, Error::SqlxError);
impl_from_error!(hyper::Error, Error::HyperError);
impl_from_error!(io::Error, Error::IOError);
impl_from_error!(AddrParseError, Error::AddrParseError);
impl_from_error!(envy::Error, Error::EnvyError);
impl_from_error!(dotenvy::Error, Error::DotEnvError);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            match self {
                Error::ConfigError(err) => err,
                Error::JWTError(err) => err.to_string(),
                Error::SqlxError(err) => err.to_string(),
                Error::HyperError(err) => err.to_string(),
                Error::EnvyError(err) => err.to_string(),
                Error::DotEnvError(err) => err.to_string(),
                Error::IOError(err) => err.to_string(),
                Error::AddrParseError(err) => err.to_string(),
            },
        )
            .into_response()
    }
}

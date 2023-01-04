use std::{io, net::AddrParseError};

macro_rules! impl_from_error {
    ($from: ty, $to: expr) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                $to(err)
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    ConfigError(String),
    HyperError(hyper::Error),
    EnvyError(envy::Error),
    IOError(io::Error),
    AddrParseError(AddrParseError),
}

impl_from_error!(hyper::Error, Error::HyperError);
impl_from_error!(io::Error, Error::IOError);
impl_from_error!(AddrParseError, Error::AddrParseError);
impl_from_error!(envy::Error, Error::EnvyError);

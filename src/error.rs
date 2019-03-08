use serde::Serialize;
use std::{error, fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    JSON(serde_json::error::Error),
    TOML(toml::de::Error),
    Actix(actix_web::Error),
    OpenSSL(openssl::error::ErrorStack),
    JWT(jsonwebtoken::errors::Error),
    AxoChat(ClientError),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::JSON(err) => Some(err),
            Error::TOML(err) => Some(err),
            Error::OpenSSL(err) => Some(err),
            Error::JWT(err) => Some(err),
            Error::AxoChat(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "I/O: {}", err),
            Error::JSON(err) => write!(f, "JSON: {}", err),
            Error::TOML(err) => write!(f, "TOML: {}", err),
            Error::Actix(err) => write!(f, "actix-web: {}", err),
            Error::OpenSSL(err) => write!(f, "OpenSSL: {}", err),
            Error::JWT(err) => write!(f, "JWT: {}", err),
            Error::AxoChat(err) => write!(f, "axochat: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::JSON(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TOML(err)
    }
}

impl From<actix_web::Error> for Error {
    fn from(err: actix_web::Error) -> Error {
        Error::Actix(err)
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(err: openssl::error::ErrorStack) -> Error {
        Error::OpenSSL(err)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Error {
        Error::JWT(err)
    }
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Error {
        Error::AxoChat(err)
    }
}

/// A client-facing error.
#[derive(Debug, Clone, Serialize)]
pub enum ClientError {
    NotSupported,
    LoginFailed,
    NotLoggedIn,
    AlreadyLoggedIn,
    MojangRequestMissing,
    RateLimited,
    PrivateMessageNotAccepted,
    EmptyMessage,
    MessageTooLong,
    InvalidCharacter(char),
    Internal,
}

impl error::Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::NotSupported => write!(f, "method not supported"),
            ClientError::LoginFailed => write!(f, "login failed"),
            ClientError::NotLoggedIn => write!(f, "not logged in"),
            ClientError::AlreadyLoggedIn => write!(f, "already logged in"),
            ClientError::MojangRequestMissing => write!(f, "mojang request missing"),
            ClientError::RateLimited => write!(f, "rate limited"),
            ClientError::PrivateMessageNotAccepted => write!(f, "private message not accepted"),
            ClientError::EmptyMessage => write!(f, "empty message"),
            ClientError::MessageTooLong => write!(f, "message was too long"),
            ClientError::InvalidCharacter(ch) => write!(f, "message contained invalid character: `{}`", ch.escape_default()),
            ClientError::Internal => write!(f, "internal error"),
        }
    }
}

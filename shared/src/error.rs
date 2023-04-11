//! This module handles shared error handling.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The shared error type that can be used by both server and client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Error)]
pub enum Error {
    /// An error occurred in accessing the database.
    #[error("error accessing the database: {0}")]
    DatabaseError(DieselError),

    /// The given password didn't match.
    #[error("invalid password")]
    InvalidPassword,

    /// An error occurred when trying to hash the user's password.
    #[error("error hashing password: {0}")]
    HashingError(String),
}

/// An error that comes from Diesel, which is used to manage the database.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Error)]
pub enum DieselError {
    /// A DB query that was meant to return something actually returned nothing.
    #[error("something in the DB was not found")]
    NotFound,

    /// A duplicate key value violates a uniqueness constraint. The values are (message, details, hint).
    #[error("duplicate key value violates unique constraint: {0}")]
    UniqueViolation(String, Option<String>, Option<String>),

    /// Some other error occurred.
    #[error("{0}")]
    Other(String),
}

#[cfg(feature = "diesel")]
mod diesel {
    use super::*;
    use ::diesel::result::{DatabaseErrorKind as Kind, Error as DsErr};
    use tracing::{debug, instrument};

    impl From<::diesel::result::Error> for DieselError {
        #[instrument(name = "from_diesel_internal_error", level = "debug")]
        fn from(value: DsErr) -> Self {
            let shared_error = match value {
                DsErr::NotFound => Self::NotFound,
                DsErr::DatabaseError(Kind::UniqueViolation, info) => {
                    let message = info.message().to_string();
                    let details = info.details().map(ToString::to_string);
                    let hint = info.hint().map(ToString::to_string);
                    debug!(?details, ?hint, "UniqueViolation: {message}");
                    Self::UniqueViolation(message, details, hint)
                }
                err => Self::Other(format!("{err:?}")),
            };
            debug!(?shared_error);
            shared_error
        }
    }
}

#[cfg(feature = "hashing")]
mod hashing {
    use super::*;

    impl From<password_hash::Error> for Error {
        fn from(value: password_hash::Error) -> Self {
            match value {
                password_hash::Error::Password => Self::InvalidPassword,
                err => Self::HashingError(format!("{err:?}")),
            }
        }
    }
}

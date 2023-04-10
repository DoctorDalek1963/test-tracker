//! This module handles shared error handling.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The shared error type that can be used by both server and client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Error)]
pub enum Error {
    /// An error occurred in accessing the database.
    #[error("error accessing the database: {0}")]
    DatabaseError(DieselError),

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

    /// Some other error occurred.
    #[error("{0}")]
    Other(String),
}

#[cfg(feature = "diesel")]
use diesel::result::Error as DsErr;
#[cfg(feature = "diesel")]
use tracing::{debug, instrument};

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for DieselError {
    #[instrument(name = "from_diesel_internal_error", level = "debug")]
    fn from(value: DsErr) -> Self {
        let shared_error = match value {
            DsErr::NotFound => Self::NotFound,
            err => Self::Other(format!("{err:?}")),
        };
        debug!(?shared_error);
        shared_error
    }
}

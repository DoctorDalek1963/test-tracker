//! This module handles hashing and verifying passwords for the database.

use crate::db::{
    establish_connection,
    models::{NewUser, User as DbUser},
};
use argon2::{
    password_hash::{
        Error as HashingError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use diesel::{result::Error as DbError, RunQueryDsl};
use test_tracker_shared::{Error as SharedError, User as SharedUser};
use thiserror::Error;
use tracing_unwrap::ResultExt;

/// Hash and salt a password for the first time. Use [`verify_password`] to verify a password
/// against the DB.
fn hash_and_salt_password(password: &str) -> Result<String, HashingError> {
    let salt: [u8; 16] = rand::random();
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(
            password.as_bytes(),
            SaltString::encode_b64(&salt)
                .expect_or_log("We should be able to encode any 16 bytes as B64")
                .as_salt(),
        )?
        .to_string())
}

/// An error that could occur when adding a new user to the database.
#[derive(Debug, Error)]
pub enum NewUserError {
    /// An error occured in accessing the DB.
    #[error("error accessing the database: {0:?}")]
    DbError(#[from] DbError),

    /// An error occured when trying to hash the password.
    #[error("unable to hash password: {0:?}")]
    HashingError(HashingError),
}

// We have to impl this by hand because `thiserror` needs its #[from] types to impl std `Error`, but
// `HashingError` doesn't
impl From<HashingError> for NewUserError {
    fn from(value: HashingError) -> Self {
        Self::HashingError(value)
    }
}

impl From<NewUserError> for SharedError {
    fn from(value: NewUserError) -> Self {
        match value {
            NewUserError::DbError(err) => SharedError::DatabaseError(err.into()),
            NewUserError::HashingError(err) => {
                SharedError::HashingError(format!("{err} ({err:?})"))
            }
        }
    }
}

/// Validate a username and password. An error means the password is invalid.
pub fn validate_user(username: &str, password: &str) -> Result<SharedUser, NewUserError> {
    use crate::db::schema::users::dsl;
    use diesel::prelude::*;

    let conn = &mut establish_connection();
    let DbUser {
        id,
        username,
        hashed_password,
    } = dsl::users
        .filter(dsl::username.eq(username.to_lowercase()))
        .first::<DbUser>(conn)?;

    let parsed_hash = PasswordHash::new(&hashed_password)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;

    Ok(SharedUser { id, username })
}

/// Add a new user to the database and return it.
pub fn add_new_user(username: &str, password: &str) -> Result<SharedUser, NewUserError> {
    use crate::db::schema::users;

    let hashed_password = hash_and_salt_password(password)?;
    let conn = &mut establish_connection();

    let user: DbUser = diesel::insert_into(users::table)
        .values(&NewUser {
            username: username.to_lowercase(),
            hashed_password,
        })
        .get_result(conn)?;

    Ok(user.into())
}

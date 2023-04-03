//! This module contains models for interacting with the DB.

use crate::db::schema::users;
use diesel::{Insertable, Queryable};
use test_tracker_shared::User as SharedUser;

/// Query a user from `users`.
#[derive(Clone, Debug, PartialEq, Queryable)]
pub struct User {
    /// The ID of the user.
    pub id: String,

    /// The username of the user.
    pub username: String,

    /// The hashed password of the user, hashed with Argon2id.
    pub hashed_password: String,
}

impl From<User> for SharedUser {
    fn from(value: User) -> Self {
        let User { id, username, .. } = value;
        Self { id, username }
    }
}

/// Insert a user into `users`.
#[derive(Clone, Debug, PartialEq, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    /// The username of the user.
    pub username: String,

    /// The hashed password of the user, hashed with Argon2id.
    pub hashed_password: String,
}

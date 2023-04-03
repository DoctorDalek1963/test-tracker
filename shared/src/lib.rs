//! This crate is a library to be shared between the client and server halves of TestTracker.

use serde::{Deserialize, Serialize};

/// A message that the client can send to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientToServerMsg {
    /// Authenticate a currently existing user with their username and password.
    Authenticate {
        /// The username of the user.
        username: String,

        /// The plaintext, unhashed password of the user.
        password: String,
    },

    /// Create a new user with the given username and password.
    CreateUser {
        /// The username of the user.
        username: String,

        /// The plaintext, unhashed password of the user.
        password: String,
    },
}

/// A message that the server can send to the client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerToClientMsg {
    /// A response to authentication.
    AuthenticationResponse(Result<User, String>),
}

/// The relevant information about a user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// A unique ID used by the server to identify the user.
    pub id: String,

    /// The user's username.
    pub username: String,
}

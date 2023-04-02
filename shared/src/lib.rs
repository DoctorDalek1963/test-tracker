//! This crate is a library to be shared between the client and server halves of TestTracker.

use serde::{Deserialize, Serialize};

/// A message that the client can send to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientToServerMsg {
    /// Authenticate a currently existing user with their username and password.
    Authenticate { username: String, password: String },
}

/// A message that the server can send to the client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerToClientMsg {
    /// A response to authentication.
    AuthenticationResponse { successful: bool, username: String },
}

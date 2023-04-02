//! This module handles interfacing with the PostgreSQL database running on the server.
//!
//! Use `$DATABASE_URL` in `/.env` to specify the URL for the database on the server.

use diesel::{Connection, PgConnection};
use std::env;

mod schema;

/// Establish a connection to the PostgreSQL database using `$DATABASE_URL`.
pub fn establish_connection() -> PgConnection {
    let database_url = env!("DATABASE_URL");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

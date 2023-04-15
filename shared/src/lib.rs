//! This crate is a library to be shared between the client and server halves of TestTracker.

pub mod error;

pub use self::error::Error;

use chrono::naive::NaiveDate;
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

    /// Get all the tests and completions for each test for the given user.
    GetTestsAndCompletions {
        /// The user's unique ID. See [`User::id`].
        user_id: String,
    },
}

/// A message that the server can send to the client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerToClientMsg {
    /// A response to authentication.
    AuthenticationResponse(Result<User, Error>),

    /// All the tests that the requested user has done, along with all the completions for each test.
    TestsAndCompletionsForUser(Result<Vec<TestAndCompletions>, Error>),
}

/// The relevant information about a user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// A unique ID used by the server to identify the user.
    pub id: String,

    /// The user's username.
    pub username: String,
}

/// The important data of the test.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TestData {
    /// The subject of the test: maths, English, science, etc.
    pub subject: String,

    /// The topic of the test: statistics, Shakespeare, organic chemistry, etc.
    pub topic: Option<String>,

    /// The date or ID of the test: Monday 3 June 2019, Mock Set 1, etc.
    pub date_or_id: String,

    /// The qualification_level of the test: GCSE, A Level, etc.
    pub qualification_level: Option<String>,

    /// The exam board for the test: Edexcel, AQA, OCR, etc.
    pub exam_board: Option<String>,

    /// A link to the paper.
    pub paper_link: Option<String>,

    /// A link to the mark scheme.
    pub mark_scheme_link: Option<String>,

    /// Any extra comments.
    pub comments: Option<String>,
}

/// The important data of the completion.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompletionData {
    /// The mark that was actually achieved.
    pub achieved_mark: i32,

    /// The total marks available.
    pub total_marks: i32,

    /// The date of the completion.
    pub date: Option<NaiveDate>,

    /// Any extra comments.
    pub comments: Option<String>,
}

/// A convenience type for a tuple containing a test and its completions.
pub type TestAndCompletions = (TestData, Vec<CompletionData>);

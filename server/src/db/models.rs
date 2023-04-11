//! This module contains models for interacting with the DB.

use crate::db::schema::{completions, tests, users};
use chrono::naive::NaiveDate;
use diesel::{Associations, Insertable, Queryable, Selectable};
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

/// Query a test from `tests`.
#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Associations)]
#[diesel(belongs_to(User))]
pub struct Test {
    /// Unique ID.
    pub id: i32,

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

    /// The ID of the user that owns this paper.
    pub user_id: String,
}

/// Query a completion from `completions`.
#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Test))]
pub struct Completion {
    /// Unique ID.
    pub id: i32,

    /// The mark that was actually achieved.
    pub achieved_mark: i32,

    /// The total marks available.
    pub total_marks: i32,

    /// The date of the completion.
    pub date: NaiveDate,

    /// Any extra comments.
    pub comments: Option<String>,

    /// The ID of the test that this completion belongs to.
    pub test_id: i32,
}

//! This module handles querying and inserting tests and completions.

use crate::db::{
    establish_connection,
    models::{Completion, Test},
    schema::{completions, tests, users},
};
use chrono::naive::NaiveDate;
use diesel::{prelude::*, result::Error};
use std::collections::HashMap;
use tracing::{instrument, trace};

/// The important data of the test.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TestData {
    /// The subject of the test: maths, English, science, etc.
    subject: String,

    /// The topic of the test: statistics, Shakespeare, organic chemistry, etc.
    topic: Option<String>,

    /// The date or ID of the test: Monday 3 June 2019, Mock Set 1, etc.
    date_or_id: String,

    /// The qualification_level of the test: GCSE, A Level, etc.
    qualification_level: Option<String>,

    /// The exam board for the test: Edexcel, AQA, OCR, etc.
    exam_board: Option<String>,
}

impl From<Test> for TestData {
    fn from(value: Test) -> Self {
        let Test {
            subject,
            topic,
            date_or_id,
            qualification_level,
            exam_board,
            ..
        } = value;

        Self {
            subject,
            topic,
            date_or_id,
            qualification_level,
            exam_board,
        }
    }
}

/// The important data of the completion.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompletionData {
    /// The mark that was actually achieved.
    achieved_mark: i32,

    /// The total marks available.
    total_marks: i32,

    /// The date of the completion.
    date: NaiveDate,

    /// Any extra comments.
    comments: Option<String>,
}

impl From<Completion> for CompletionData {
    fn from(value: Completion) -> Self {
        let Completion {
            achieved_mark,
            total_marks,
            date,
            comments,
            ..
        } = value;

        Self {
            achieved_mark,
            total_marks,
            date,
            comments,
        }
    }
}

/// For the given user, find all the tests they own and all the completions that each of those
/// tests have.
#[instrument]
pub fn get_all_tests_and_completions_for_user(
    user_id: &str,
) -> Result<Vec<(TestData, Vec<CompletionData>)>, Error> {
    let tests_and_completions: Vec<(Test, Completion)> = users::table
        .inner_join(tests::table.inner_join(completions::table))
        .filter(users::id.eq(user_id))
        .select((Test::as_select(), Completion::as_select()))
        .load::<(Test, Completion)>(&mut establish_connection())?;
    trace!(?tests_and_completions);

    let mut map: HashMap<TestData, Vec<CompletionData>> = HashMap::new();

    for (test, completion) in
        tests_and_completions
            .into_iter()
            .map(|(test, completion)| -> (TestData, CompletionData) {
                (test.into(), completion.into())
            })
    {
        match map.get_mut(&test) {
            None => {
                map.insert(test, vec![completion]);
            }
            Some(completions) => completions.push(completion),
        };
    }

    Ok(map.into_iter().collect())
}

//! This module handles querying and inserting tests and completions.

use crate::db::{
    establish_connection,
    models::{Completion, Test},
    schema::{completions, tests, users},
};
use diesel::{prelude::*, result::Error};
use std::collections::HashMap;
use test_tracker_shared::{CompletionData, TestAndCompletions, TestData};
use tracing::{instrument, trace};

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
) -> Result<Vec<TestAndCompletions>, Error> {
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

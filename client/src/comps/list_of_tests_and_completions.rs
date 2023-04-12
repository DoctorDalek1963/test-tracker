//! This module provides the [`ListOfTestsAndCompletions`] component.

use crate::comps::TestAndCompletions;
use test_tracker_shared::TestAndCompletions as SharedTAC;
use yew::{function_component, html, Html, Properties};

/// The props for [`ListOfTestsAndCompletions`].
#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    /// The list of tests and completions.
    pub list: Vec<SharedTAC>,
}

/// The component to render a list of tests and completions. See [`TestAndCompletions`] for an
/// individual one.
#[function_component(ListOfTestsAndCompletions)]
pub fn list_of_tests_and_completions(Props { list }: &Props) -> Html {
    list.iter()
        .map(|data| {
            html! {
                <TestAndCompletions test_and_completions={data.clone()} />
            }
        })
        .collect()
}

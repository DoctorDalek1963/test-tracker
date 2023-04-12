//! This module provides the [`TestAndCompletions`] component.

use test_tracker_shared::TestAndCompletions as SharedTAC;
use yew::{function_component, html, Html, Properties};

/// The props for [`TestAndCompletions`].
#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    /// The test and completions to be rendered by this component.
    pub test_and_completions: SharedTAC,
}

/// The component to a render an individual test with its completions.
#[function_component(TestAndCompletions)]
pub fn test_and_completion(
    Props {
        test_and_completions: (test, completions),
    }: &Props,
) -> Html {
    // TODO: Add real code here
    html! {
        <>
        <p> {format!("{test:?}")} </p>
        <p> {format!("{completions:?}")} </p>
        </>
    }
}

//! This module provides the [`TestAndCompletions`] component.

use crate::comps::Completion;
use test_tracker_shared::{TestAndCompletions as SharedTAC, TestData};
use url::Url;
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
    let TestData {
        subject,
        topic,
        date_or_id,
        qualification_level,
        exam_board,
        paper_link,
        mark_scheme_link,
        comments,
    } = test.clone();

    let completions: Html = completions
        .iter()
        .map(|data| html! { <Completion data={data.clone()} /> })
        .collect();

    html! {
        <div class="test">
            <div class="title">
                <span class="subject"> { subject } </span>
                if let Some(topic) = topic {
                    <span class="topic"> { format!(": {topic}") } </span>
                }
            </div>
            <div class="content">
                <div class="date-or-id"> { date_or_id } </div>
                if let Some(qual) = qualification_level {
                    <div class="qualification-level"> { qual } </div>
                }
                if let Some(board) = exam_board {
                    <div class="exam-board"> { board } </div>
                }
                if let Some(link) = paper_link {
                    <div class="paper-link"> { "Paper: " }
                        if Url::parse(&link).is_ok() {
                            <a href={link.clone()}> { link } </a>
                        } else {
                            { link }
                        }
                    </div>
                }
                if let Some(link) = mark_scheme_link {
                    <div class="mark-scheme-link"> { "Mark scheme: " }
                        if Url::parse(&link).is_ok() {
                            <a href={link.clone()}> { link } </a>
                        } else {
                            { link }
                        }
                    </div>
                }
                if let Some(comments) = comments {
                    <div class="comments"> { comments } </div>
                }

                <div class="completions-list">
                    {completions}
                </div>
            </div>
        </div>
    }
}

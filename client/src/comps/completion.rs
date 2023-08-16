//! This module provides the [`Completion`] component.

use test_tracker_shared::CompletionData;
use yew::{function_component, html, Html, Properties};

/// The props for [`Completion`].
#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub data: CompletionData,
}

/// The component to render an individual component.
#[function_component(Completion)]
pub fn completion(Props { data }: &Props) -> Html {
    let CompletionData {
        achieved_mark,
        total_marks,
        date,
        comments,
    } = data.clone();

    html! {
        <div class="completion">
            <div class="marks">
                <span class="achieved-mark"> { achieved_mark } </span>
                <span class="slash"> { " / " } </span>
                <span class="total-marks"> { total_marks } </span>
            </div>
            if let Some(date) = date {
                <div class="date"> { date } </div>
            }
            if let Some(comments) = comments {
                <div class="comments"> { comments } </div>
            }
        </div>
    }
}

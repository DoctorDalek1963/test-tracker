//! This module provides a component for an error message box.

use yew::{html, Component, Context, Properties};

/// The error message component itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErrorMessage;

/// The properties of the error message box.
#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    /// The message to give to the error message.
    pub msg: String,
}

impl Component for ErrorMessage {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let msg = ctx.props().msg.clone();
        html! {
            <div class="error-message">
                <h4> {"ERROR:"} </h4>
                {msg}
            </div>
        }
    }
}

//! This module handles the login form.

use tracing::{instrument, trace};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_state, Callback, Html, Properties};

/// Get the text value from the given input event.
#[instrument]
fn get_value_from_input_event(event: yew::Event) -> String {
    let event: web_sys::Event = event.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();

    let value = target.value();
    trace!(?event, ?value);
    value
}

/// The props to use for the login form.
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    /// The callback to run when th login form is submitted. It takes the username and password as
    /// arguments.
    pub onsubmit: Callback<(String, String)>,
}

/// Provide a login form prompting for username and password.
#[function_component(LoginForm)]
pub fn login_form(props: &Props) -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());

    let on_username_changed = {
        let username = username.clone();
        move |event: yew::Event| username.set(get_value_from_input_event(event))
    };
    let on_password_changed = {
        let password = password.clone();
        move |event: yew::Event| password.set(get_value_from_input_event(event))
    };

    let onclick = {
        let props = props.clone();
        move |_mouse_event| {
            props
                .onsubmit
                .emit((username.to_string(), password.to_string()));
        }
    };

    html! {
        <div class="login-form">
            <label> {"Username"} </label>
            <input type="text" name="username" onchange={on_username_changed} />
            <label> {"Password"} </label>
            <input type="text" name="password" onchange={on_password_changed} />
            <button {onclick}> {"Submit"} </button>
        </div>
    }
}

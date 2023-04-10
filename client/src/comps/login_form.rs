//! This module handles the login form.

use crate::comps::error_message::ErrorMessage;
use derive_more::From;
use tracing::{instrument, trace};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::{
    classes, function_component, html, use_state, Callback, Component, Context, Html, Properties,
};

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

/// The props for the [`LoginOrCreateAccountForm`].
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LoginOrCreateAccountProps {
    /// The callback to run when the user tries to login. Takes username and password.
    pub onsubmit_login: Callback<(String, String)>,

    /// The callback to run when the user tries to create a new account. Takes username and password.
    pub onsubmit_create_account: Callback<(String, String)>,

    pub invalid_username_or_password: bool,
}

/// The tabs for logging in or creating a new account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginOrCreateAccountTab {
    /// Login.
    Login,

    /// Create a new account.
    CreateAccount,
}

#[derive(Clone, Debug, Eq, PartialEq, From)]
pub enum LoginOrCreateAccountMsg {
    ChangeTab(LoginOrCreateAccountTab),
    ChangeError(Option<String>),
}

/// A component to manage logging in and creating accounts, with the options presented in tabs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginOrCreateAccountForm {
    /// Which tab is currently selected.
    tab: LoginOrCreateAccountTab,

    /// A possible error to show.
    error: Option<String>,
}

impl LoginOrCreateAccountForm {
    /// View the login tab.
    fn view_login_tab(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.props().onsubmit_login.clone();
        html! {
            <InternalLoginForm {onsubmit} title={"Login"} />
        }
    }

    /// View the create account tab.
    fn view_create_account_tab(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.props().onsubmit_create_account.clone();
        html! {
            <InternalLoginForm {onsubmit} title={"Create account"} />
        }
    }
}

impl Component for LoginOrCreateAccountForm {
    type Message = LoginOrCreateAccountMsg;
    type Properties = LoginOrCreateAccountProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            tab: LoginOrCreateAccountTab::Login,
            error: match ctx.props().invalid_username_or_password {
                true => Some("Invalid username or password".to_string()),
                false => None,
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let form = match self.tab {
            LoginOrCreateAccountTab::Login => self.view_login_tab(ctx),
            LoginOrCreateAccountTab::CreateAccount => self.view_create_account_tab(ctx),
        };

        let login_tab = ctx
            .link()
            .callback(|_event| LoginOrCreateAccountMsg::ChangeTab(LoginOrCreateAccountTab::Login));
        let create_account_tab = ctx.link().callback(|_event| {
            LoginOrCreateAccountMsg::ChangeTab(LoginOrCreateAccountTab::CreateAccount)
        });

        let (login_selected, create_account_selected) = match self.tab {
            LoginOrCreateAccountTab::Login => (Some("selected"), None),
            LoginOrCreateAccountTab::CreateAccount => (None, Some("selected")),
        };

        let error_message = match &self.error {
            Some(msg) => html! {
                <ErrorMessage msg={msg.clone()} />
            },
            None => html! {},
        };

        html! {
            <>
            <div class="login-or-create-account-form">
                <div class="tabs">
                    <button
                        class={classes!("tab", login_selected)}
                        onclick={login_tab}
                    >
                        { "Login" }
                    </button>
                    <button
                        class={classes!("tab", create_account_selected)}
                        onclick={create_account_tab}
                    >
                        { "Create account" }
                    </button>
                </div>

                {form}
            </div>

            {error_message}
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginOrCreateAccountMsg::ChangeTab(tab) => {
                self.tab = tab;
                self.error = None;
                true
            }
            LoginOrCreateAccountMsg::ChangeError(error) => {
                self.error = error;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().invalid_username_or_password {
            self.error = Some("Invalid username or password".to_string());
            true
        } else {
            false
        }
    }
}

/// The props for the [`InternalLoginForm`].
#[derive(Clone, Debug, PartialEq, Properties)]
struct InternalLoginProps {
    /// The callback to run when the login form is submitted. It takes the username and password as
    /// arguments.
    onsubmit: Callback<(String, String)>,

    /// The title of this form.
    title: String,
}

/// An implementation detail for ease of creating login-like forms.
#[function_component(InternalLoginForm)]
fn internal_login_form(props: &InternalLoginProps) -> Html {
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
        <div class="form">
            <h3> {props.title.clone()} </h3>

            <div class="title-and-input">
                <label for="usernameBox"> {"Username"} </label>
                <input id="usernameBox" type="text" name="username" onchange={on_username_changed} /><br/>
            </div>
            <div class="title-and-input">
                <label for="passwordBox"> {"Password"} </label>
                <input id="passwordBox" type="password" name="password" onchange={on_password_changed} /><br/>
            </div>

            <button {onclick}> {"Submit"} </button>
        </div>
    }
}

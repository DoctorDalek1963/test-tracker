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

/// A callback to run when the user tries to login or create an account. It takes username,
/// password, "remember me".
pub type LoginOrCreateAccountCallback = Callback<(String, String, bool)>;

/// The props for the [`LoginOrCreateAccountForm`].
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LoginOrCreateAccountProps {
    /// The callback for logging in.
    pub onsubmit_login: LoginOrCreateAccountCallback,

    /// The callback for creating a new account.
    pub onsubmit_create_account: LoginOrCreateAccountCallback,

    /// Did the user recently enter an invalid username or password?
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

/// A message type for [`LoginOrCreateAccountForm`] to use.
#[derive(Clone, Debug, Eq, PartialEq, From)]
pub enum LoginOrCreateAccountMsg {
    /// Change to the specified tab.
    ChangeTab(LoginOrCreateAccountTab),

    /// Change (or clear) the current error message.
    ChangeError(Option<String>),

    /// Submit a login or create account request with the given parameters.
    Submit(LoginOrCreateAccountTab, (String, String, bool)),
}

/// A component to manage logging in and creating accounts, with the options presented in tabs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginOrCreateAccountForm {
    /// Which tab is currently selected.
    tab: LoginOrCreateAccountTab,

    /// A possible error to show.
    error: Option<String>,
}

/// Create a callback for the given tab. This callback sends a [`LoginOrCreateAccountMsg`] to
/// [`LoginOrCreateAccountForm`] to submit the prop callback or display an appropriate error.
fn create_onsubmit_callback(
    ctx: &Context<LoginOrCreateAccountForm>,
    tab: LoginOrCreateAccountTab,
) -> LoginOrCreateAccountCallback {
    ctx.link().callback(
        move |(username, password, remember_me): (String, String, bool)| {
            if username.is_empty() || password.is_empty() {
                LoginOrCreateAccountMsg::ChangeError(Some(
                    "Please enter a username and password".to_string(),
                ))
            } else {
                LoginOrCreateAccountMsg::Submit(tab, (username, password, remember_me))
            }
        },
    )
}

impl LoginOrCreateAccountForm {
    /// View the login tab.
    fn view_login_tab(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = create_onsubmit_callback(ctx, LoginOrCreateAccountTab::Login);
        html! {
            <InternalLoginForm {onsubmit} title={"Login"} />
        }
    }

    /// View the create account tab.
    fn view_create_account_tab(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = create_onsubmit_callback(ctx, LoginOrCreateAccountTab::CreateAccount);
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            LoginOrCreateAccountMsg::Submit(tab, params) => {
                match tab {
                    LoginOrCreateAccountTab::Login => ctx.props().onsubmit_login.emit(params),
                    LoginOrCreateAccountTab::CreateAccount => {
                        ctx.props().onsubmit_create_account.emit(params);
                    }
                };
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
    onsubmit: LoginOrCreateAccountCallback,

    /// The title of this form.
    title: String,
}

/// An implementation detail for ease of creating login-like forms.
#[function_component(InternalLoginForm)]
fn internal_login_form(props: &InternalLoginProps) -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let remember_me = use_state(|| false);

    let on_username_changed = {
        let username = username.clone();
        move |event: yew::Event| username.set(get_value_from_input_event(event))
    };
    let on_password_changed = {
        let password = password.clone();
        move |event: yew::Event| password.set(get_value_from_input_event(event))
    };
    let on_checkbox_changed = {
        let remember_me = remember_me.clone();
        move |_event| remember_me.set(!*remember_me)
    };

    let onclick = {
        let props = props.clone();
        move |_mouse_event| {
            props
                .onsubmit
                .emit((username.to_string(), password.to_string(), *remember_me));
        }
    };

    html! {
        <div class="form">
            <h3> {props.title.clone()} </h3>

            <div class="label-and-input-box">
                <label for="usernameBox"> {"Username"} </label>
                <input id="usernameBox" type="text" name="username" onchange={on_username_changed} /><br/>
            </div>
            <div class="label-and-input-box">
                <label for="passwordBox"> {"Password"} </label>
                <input id="passwordBox" type="password" name="password" onchange={on_password_changed} /><br/>
            </div>

            <div class="label-and-checkbox">
                <input
                    type="checkbox"
                    id="remember-me"
                    aria-label="Remember me"
                    value="Remember me"
                    onchange={on_checkbox_changed} />
                <label for="remember-me"> { "Remember me" } </label>
            </div>

            <button {onclick}> {"Submit"} </button>
        </div>
    }
}

//! This crate provides a web interface to TestTracker.

#![feature(min_specialization)]

use self::{
    comps::{error_message::ErrorMessage, login_form::LoginOrCreateAccountForm, navbar::Navbar},
    web::{get_user, local_storage, session_storage},
};
use gloo_utils::window;
use lazy_static::lazy_static;
use reqwest_wasm::Client;
use std::{error::Error, sync::Arc};
use test_tracker_shared::{
    error::DieselError as SharedDieselError, ClientToServerMsg, Error as SharedError,
    ServerToClientMsg, User,
};
use tracing::{debug, error, info, instrument, warn};
use tracing_unwrap::ResultExt;
use tracing_wasm::WASMLayerConfigBuilder;
use yew::{html, Component, Context, Html};

mod comps;
mod web;

/// The key for the dark mode key in browser storage.
pub(crate) const STORAGE_KEY_DARK_MODE: &str = "testTrackerDarkMode";

/// The key for the user key in browser storage.
pub(crate) const STORAGE_KEY_USER: &str = "testTrackerUser";

lazy_static! {
    /// The client to use for making async requests to the server.
    static ref REQWEST_CLIENT: Arc<Client> = Arc::new(Client::new());
}

/// The model for the whole web app.
#[derive(Clone, Debug)]
struct App {
    /// The user that we may or may not have authenticated.
    user: Option<User>,

    /// An optional error message to display.
    error_message: Option<String>,
}

/// A message to send to the app.
#[derive(Debug)]
enum AppMsg {
    /// Manually change or reset the internal [`error_message`].
    ChangeErrorMessage(Option<String>),

    /// An error from the server has occured. Tell the user.
    SharedError(SharedError),

    /// An unknown error has occured. Tell the user.
    UnknownError(Box<dyn Error>),

    /// Authenticate a user. The bool reflects the "remember me" checkbox.
    AuthenticateUser(User, bool),
}

impl<E: Error + 'static> From<E> for AppMsg {
    default fn from(value: E) -> Self {
        Self::UnknownError(Box::new(value))
    }
}

impl From<SharedError> for AppMsg {
    fn from(value: SharedError) -> Self {
        Self::SharedError(value)
    }
}

impl App {
    #[instrument(skip_all)]
    fn view_login_screen(&self, ctx: &Context<Self>) -> Html {
        /// Generate an `onsubmit` callback for logging in or creating an account.
        macro_rules! onsubmit_login_or_create_account {
            ($message:ident) => {
                ctx.link().callback_future(move |(username, password, remember_me): (String, String, bool)| {
                    let client = Arc::clone(&REQWEST_CLIENT);

                    debug!(
                        ?username, ?password, ?remember_me,
                        concat!("Trying to authenticate with ", stringify!($message))
                    );

                    // This async block messages the server to try to authenticate a user
                    async move {
                        if username.is_empty() || password.is_empty() {
                            return AppMsg::ChangeErrorMessage(
                                Some("Please enter a username or password".to_string())
                            );
                        }

                        debug!("Sending auth request to server");
                        match client
                            .post(env!("SERVER_URL"))
                            .body(
                                ron::to_string(&ClientToServerMsg::$message {
                                    username,
                                    password,
                                })
                                .expect_or_log(
                                    "Converting a ClientToServerMsg to a RON string shouldn't fail",
                                ),
                            )
                            .send()
                            .await
                        {
                            Ok(response) => {
                                debug!(?response);
                                let text = response.text().await;
                                debug!(?text);

                                match text {
                                    Ok(body) => {
                                        let msg = ron::from_str(&body);
                                        debug!(?msg);
                                        match msg {
                                            Ok(msg) => match msg {
                                                ServerToClientMsg::AuthenticationResponse(result) => match result {
                                                    Ok(user) => AppMsg::AuthenticateUser(user, remember_me),
                                                    Err(e) => e.into()
                                                }
                                            },
                                            Err(e) => e.into(),
                                        }
                                    }
                                    Err(e) => e.into(),
                                }
                            }
                            Err(e) => e.into(),
                        }
                    }
                })
            };
        }

        let onsubmit_login = onsubmit_login_or_create_account!(Authenticate);
        let onsubmit_create_account = onsubmit_login_or_create_account!(CreateUser);

        let error_message = match &self.error_message {
            Some(msg) => html! {
                <ErrorMessage msg={msg.clone()} />
            },
            None => html! {},
        };

        html! {
            <>
            <LoginOrCreateAccountForm
                {onsubmit_login}
                {onsubmit_create_account} />
            {error_message}
            </>
        }
    }

    #[instrument(skip_all)]
    fn view_main_screen(&self, _ctx: &Context<Self>, user: &User) -> Html {
        html! {
            <p> { format!("Logged in as {}", user.username) } </p>
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            user: get_user(),
            error_message: None,
        }
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = match self.user {
            Some(ref user) => self.view_main_screen(ctx, user),
            None => self.view_login_screen(ctx),
        };

        html! {
            <>
            <Navbar />
            <div id="content">
                {content}
            </div>
            </>
        }
    }

    #[instrument(skip_all)]
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        debug!(?msg);
        match msg {
            AppMsg::AuthenticateUser(user, remember_me) => {
                let user_str = ron::to_string(&user)
                    .expect_or_log("We should be able to serialize a User to a String");

                local_storage()
                    .set_item(STORAGE_KEY_USER, &user_str)
                    .expect_or_log(
                        "We should be able to set a localStorage value without a problem",
                    );

                if remember_me {
                    session_storage()
                        .set_item(STORAGE_KEY_USER, &user_str)
                        .expect_or_log(
                            "We should be able to set a sessionStorage value without a problem",
                        );
                }

                self.user = Some(user);
                self.error_message = None;

                true
            }
            AppMsg::ChangeErrorMessage(msg) => {
                self.error_message = msg;
                true
            }
            AppMsg::SharedError(error) => match error {
                SharedError::DatabaseError(SharedDieselError::NotFound)
                | SharedError::InvalidPassword => {
                    warn!("Invalid username or password");
                    self.error_message = Some("Invalid username or password".to_string());
                    true
                }
                SharedError::DatabaseError(SharedDieselError::UniqueViolation(_, details, _))
                    if details.clone().is_some_and(|s| s.contains("username")) =>
                {
                    warn!("Username already taken");
                    self.error_message = Some("Username already taken".to_string());
                    true
                }
                e => {
                    error!(?e);
                    window()
                        .alert_with_message(&format!("Error: {e:?}"))
                        .expect_or_log("Error alerting the user");
                    true
                }
            },
            AppMsg::UnknownError(error) => {
                error!(?error, "Unknown error");
                window()
                    .alert_with_message(&format!("Unknown error: {error:?}"))
                    .expect_or_log("Error alerting the user");
                true
            }
        }
    }
}

/// Set things up and start the app.
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        #[cfg(debug_assertions)]
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::DEBUG)
            .build(),
        #[cfg(not(debug_assertions))]
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    info!("Starting app");
    yew::Renderer::<App>::new().render();
}

//! This crate provides a web interface to TestTracker.

#![feature(min_specialization)]

use self::{
    comps::{ErrorMessage, ListOfTestsAndCompletions, LoginOrCreateAccountForm, Navbar},
    web::{get_user, local_storage, session_storage},
};
use gloo_utils::window;
use lazy_static::lazy_static;
use reqwest_wasm::Client;
use std::{error::Error, sync::Arc};
use test_tracker_shared::{
    error::DieselError as SharedDieselError, ClientToServerMsg, Error as SharedError,
    ServerToClientMsg, TestAndCompletions, User,
};
use tracing::{debug, error, info, instrument, trace, warn};
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

    /// The tests and completions of the user.
    tests_and_completions: Vec<TestAndCompletions>,

    /// An optional error message to display.
    error_message: Option<String>,
}

/// A message to send to the app.
#[derive(Debug)]
enum AppMsg {
    /// Manually change or reset the internal [`error_message`](App::error_message).
    ChangeErrorMessage(Option<String>),

    /// An error from the server has occured. Tell the user.
    SharedError(SharedError),

    /// An unknown error has occured. Tell the user.
    UnknownError(Box<dyn Error>),

    /// We received an unexpected (but valid) message from the server.
    UnexpectedServerMsg(ServerToClientMsg),

    /// Authenticate a user. The bool reflects the "remember me" checkbox.
    AuthenticateUser(User, bool),

    /// Set the list of tests and completions.
    SetTestsAndCompletionsList(Vec<TestAndCompletions>),
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

/// Send a message to the server.
///
/// This returns a callback future from the given context, which sends a message to the server and
/// reacts accordingly.
///
/// `ctx` is the yew context to create the callback future with.
/// `args` is the arguments (in a tuple) to the callback.
/// `args_type` is the type of the arguments tuple.
/// `pre_send` is any code that you want to execute before sending the message. This code could
/// early return if you wanted to check that a password was non-empty, for example.
/// `msg` is the message to send to the server.
/// `expected_result => reaction` is the "happy path" of the match pattern, where you get the
/// response that you were expecting. It matches against a [`ServerToClientMsg`].
macro_rules! send_message_to_server {
    (
        $ctx:expr;
        |$args:tt: $args_type:ty|;
        $pre_send:block;
        $msg:expr;
        $expected_result:pat => $reaction:expr
    ) => {
        $ctx.link().callback_future(move |$args: $args_type| {
            let client = Arc::clone(&REQWEST_CLIENT);

            async move {
                $pre_send;

                match client
                    .post(env!("SERVER_URL"))
                    .body(ron::to_string(&$msg).expect_or_log(
                        "Converting a ClientToServerMsg to a RON string shouldn't fail",
                    ))
                    .send()
                    .await
                {
                    Ok(response) => {
                        trace!(?response, "Received raw response from server");
                        let text = response.text().await;
                        trace!(?text, "Got text from server response");

                        match text {
                            Ok(body) => {
                                let msg = ron::from_str(&body);
                                trace!(?msg, "Deserialized msg from server response");
                                match msg {
                                    Ok(msg) => match msg {
                                        $expected_result => $reaction,
                                        msg => AppMsg::UnexpectedServerMsg(msg),
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

impl App {
    /// Get the HTML for the login screen.
    #[instrument(skip_all)]
    fn view_login_screen(&self, ctx: &Context<Self>) -> Html {
        /// Generate an `onsubmit` callback for logging in or creating an account.
        macro_rules! onsubmit_login_or_create_account {
            ($message:ident) => {
                send_message_to_server! {
                    ctx;
                    |(username, password, remember_me): (String, String, bool)|;
                    {
                        debug!(
                            ?username, ?password, ?remember_me,
                            concat!("Trying to authenticate with ", stringify!($message))
                        );

                        if username.is_empty() || password.is_empty() {
                            return AppMsg::ChangeErrorMessage(
                                Some("Please enter a username or password".to_string())
                            );
                        }
                    };
                    ClientToServerMsg::$message { username, password };
                    ServerToClientMsg::AuthenticationResponse(result) => match result {
                        Ok(user) => AppMsg::AuthenticateUser(user, remember_me),
                        Err(e) => e.into(),
                    }
                }
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

    /// Get the HTML for the main screen.
    #[instrument(skip_all)]
    fn view_main_screen(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ListOfTestsAndCompletions list={self.tests_and_completions.clone()} />
        }
    }

    /// Refresh the internal [`tests_and_completions`](App::tests_and_completions) attribute by
    /// creating an async callback to get the list from the server and send the
    /// [`SetTestsAndCompletionsList`](AppMsg::SetTestsAndCompletionsList) message to the app.
    fn refresh_tests_and_completions_list(&self, ctx: &Context<Self>) {
        match &self.user {
            Some(user) => send_message_to_server! {
                ctx;
                |user_id: String|;
                {};
                ClientToServerMsg::GetTestsAndCompletions { user_id };
                ServerToClientMsg::TestsAndCompletionsForUser(result) => match result {
                    Ok(tests_and_completions) => {
                        debug!(?tests_and_completions);
                        AppMsg::SetTestsAndCompletionsList(tests_and_completions)
                    }
                    Err(e) => e.into(),
                }
            }
            .emit(user.id.clone()),
            None => {
                panic!("Cannot refresh tests_and_completions list until the user has logged in")
            }
        };
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            user: get_user(),
            tests_and_completions: vec![],
            error_message: None,
        }
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let app = Self::default();
        // If the user is logged in from last time, then initiate the
        // async callback to refresh the list
        if app.user.is_some() {
            app.refresh_tests_and_completions_list(ctx);
        }
        app
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = match self.user {
            Some(_) => self.view_main_screen(ctx),
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
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        trace!(?msg, "Updating in reponse to message");
        match msg {
            AppMsg::AuthenticateUser(user, remember_me) => {
                let user_str = ron::to_string(&user)
                    .expect_or_log("We should be able to serialize a User to a String");

                session_storage()
                    .set_item(STORAGE_KEY_USER, &user_str)
                    .expect_or_log(
                        "We should be able to set a sessionStorage value without a problem",
                    );

                if remember_me {
                    local_storage()
                        .set_item(STORAGE_KEY_USER, &user_str)
                        .expect_or_log(
                            "We should be able to set a localStorage value without a problem",
                        );
                }

                self.user = Some(user);
                self.error_message = None;

                self.tests_and_completions = vec![];
                self.refresh_tests_and_completions_list(ctx);

                true
            }
            AppMsg::SetTestsAndCompletionsList(list) => {
                self.tests_and_completions = list;
                true
            }
            AppMsg::ChangeErrorMessage(msg) => {
                self.error_message = msg;
                true
            }
            AppMsg::UnexpectedServerMsg(msg) => {
                self.error_message = Some(format!("{msg:?}"));
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

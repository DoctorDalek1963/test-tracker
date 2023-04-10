//! This crate provides a web interface to TestTracker.

use self::comps::{login_form::LoginOrCreateAccountForm, navbar::Navbar};
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

lazy_static! {
    /// The client to use for making async requests to the server.
    static ref REQWEST_CLIENT: Arc<Client> = Arc::new(Client::new());
}

/// The model for the whole web app.
#[derive(Clone, Debug)]
struct App {
    /// The user that we may or may not have authenticated.
    user: Option<User>,

    /// Did the user recently enter an invalid username or password?
    invalid_username_or_password: bool,
}

/// A message to send to the app.
#[derive(Debug)]
enum AppMsg {
    /// An error has occured. Tell the user.
    Error(Box<dyn Error>),

    /// We have received a message from the server.
    ServerMsg(ServerToClientMsg),
}

impl<E: Error + 'static> From<E> for AppMsg {
    fn from(value: E) -> Self {
        Self::Error(Box::new(value))
    }
}

impl App {
    #[instrument(skip_all)]
    fn view_login_screen(&self, ctx: &Context<Self>) -> Html {
        /// Generate an `onsubmit` callback for logging in or creating an account.
        macro_rules! onsubmit_login_or_create_account {
            ($message:ident) => {
                ctx.link().callback_future(move |(username, password, remember_me)| {
                    let client = Arc::clone(&REQWEST_CLIENT);

                    debug!(
                        ?username, ?password, ?remember_me,
                        concat!("Authenticating with ", stringify!($message))
                    );

                    // This async block messages the server to try to authenticate a user
                    async move {
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
                                            Ok(msg) => AppMsg::ServerMsg(msg),
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

        html! {
            <LoginOrCreateAccountForm
                {onsubmit_login}
                {onsubmit_create_account}
                invalid_username_or_password={self.invalid_username_or_password} />
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
            user: None,
            invalid_username_or_password: false,
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
            AppMsg::Error(e) => {
                error!(?e, "AppMsg::Error");
                window()
                    .alert_with_message(&format!("Error: {e:?}"))
                    .expect_or_log("Error alerting the user");
                true
            }
            AppMsg::ServerMsg(msg) => match msg {
                ServerToClientMsg::AuthenticationResponse(result) => match result {
                    Ok(user) => {
                        self.user = Some(user);
                        self.invalid_username_or_password = false;
                        true
                    }
                    Err(
                        SharedError::DatabaseError(SharedDieselError::NotFound)
                        | SharedError::InvalidPassword,
                    ) => {
                        warn!("Invalid username or password");
                        self.invalid_username_or_password = true;
                        true
                    }
                    Err(e) => {
                        error!(?e, "Unknown error authenticating user");
                        window()
                            .alert_with_message(&format!(
                                "Unknown error authenticating user: {e:?}"
                            ))
                            .expect_or_log("Error alerting the user");
                        true
                    }
                },
            },
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

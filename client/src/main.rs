//! This crate provides a web interface to TestTracker.

use reqwest_wasm::Client;
use std::{error::Error, sync::Arc};
use test_tracker_shared::{ClientToServerMsg, ServerToClientMsg};
use tracing::{debug, info, instrument};
use tracing_unwrap::ResultExt;
use tracing_wasm::WASMLayerConfigBuilder;
use yew::{html, Component, Html};

/// The model for the whole web app.
#[derive(Clone, Debug)]
struct App {
    /// The main text.
    text: String,

    /// The client to use for requests.
    reqwest_client: Arc<Client>,

    /// Whether we have authenticated a user.
    authenticated: bool,
}

/// A message to send to the app.
#[derive(Debug)]
enum AppMsg {
    /// Do nothing.
    Noop,

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

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            text: "Hello world!".to_string(),
            reqwest_client: Arc::new(Client::new()),
            authenticated: false,
        }
    }

    #[instrument(skip_all)]
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let client = self.reqwest_client.clone();
        let authenticated = self.authenticated;

        // This async block messages the server to try to authenticate a user
        ctx.link().send_future(async move {
            if authenticated {
                return Self::Message::Noop;
            }

            debug!("Sending auth request to server");
            match client
                .post(env!("SERVER_URL"))
                .body(
                    ron::to_string(&ClientToServerMsg::Authenticate {
                        username: "dyson".to_string(),
                        password: "adminpass".to_string(),
                    })
                    .expect_or_log("Converting a ClientToServerMsg to a RON string shouldn't fail"),
                )
                .send()
                .await
            {
                Ok(response) => {
                    debug!(?response);
                    match response.text().await {
                        Ok(body) => match ron::from_str(&body) {
                            Ok(msg) => Self::Message::ServerMsg(msg),
                            Err(e) => e.into(),
                        },
                        Err(e) => e.into(),
                    }
                }
                Err(e) => e.into(),
            }
        });

        html! {
            <p> { self.text.clone() } </p>
        }
    }

    #[instrument(skip_all)]
    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        debug!(?msg);
        match msg {
            AppMsg::Noop => false,
            AppMsg::Error(e) => {
                self.text = format!("ERROR: {e:?}");
                true
            }
            AppMsg::ServerMsg(msg) => match msg {
                ServerToClientMsg::AuthenticationResponse {
                    successful,
                    username,
                } => {
                    if successful {
                        self.authenticated = true;
                        self.text = format!("Successfully authenticated {username}");
                        true
                    } else {
                        self.authenticated = false;
                        self.text = format!("Failed to authenticate {username}");
                        true
                    }
                }
            },
        }
    }
}

/// Set things up and start the app.
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::DEBUG)
            .build(),
    );

    info!("Starting app");
    yew::Renderer::<App>::new().render();
}

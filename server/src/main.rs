//! This is the server for TestTracker, which uses a PostgreSQL database to store all the data.

use color_eyre::Result;
use std::io;
use test_tracker_shared::{ClientToServerMsg, ServerToClientMsg};
use tiny_http::{Header, Request, Response};
use tracing::{info, instrument};
use tracing_unwrap::ResultExt;

mod db;

/// Handle a single HTTP request.
#[instrument(skip_all, fields(addr = ?req.remote_addr()))]
async fn handle_request(mut req: Request) -> Result<()> {
    info!("Received a new request");

    let mut body = String::new();
    req.as_reader().read_to_string(&mut body)?;
    let msg: ClientToServerMsg = ron::from_str(&body)?;

    match msg {
        ClientToServerMsg::Authenticate { username, password } => {
            info!(?username, ?password, "Trying to authenticate");
            req.respond(
                Response::from_string(
                    ron::to_string(&ServerToClientMsg::AuthenticationResponse {
                        successful: true,
                        username,
                    })
                    .unwrap(),
                )
                // Tell CORS to shut up
                .with_header(Header {
                    field: "Access-Control-Allow-Origin".parse().unwrap(),
                    value: "*".parse().unwrap(),
                }),
            )?;
        }
    };

    Ok(())
}

/// Setup the global tracing subscriber to send log messages to stdout, and a `server.log` file
/// (rotated daily).
fn setup_global_tracing_subscriber() {
    use tracing_subscriber::{fmt::Layer, prelude::*};

    let appender = tracing_appender::rolling::daily(env!("SERVER_LOG_PATH"), "server.log");

    let subscriber = tracing_subscriber::registry()
        .with(Layer::new().with_writer(appender).with_ansi(false))
        .with(Layer::new().with_writer(std::io::stdout).with_ansi(true));

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting the global default for tracing should be okay");
}

/// Create and run the server indefinitely.
#[tokio::main]
#[instrument]
async fn main() -> io::Result<()> {
    setup_global_tracing_subscriber();

    info!(port = env!("PORT"), "Initialising server");

    let server = tiny_http::Server::http(concat!("localhost:", env!("PORT")))
        .expect_or_log("Creating the server should not fail");
    let _conn = db::establish_connection();

    info!("Server initialised");

    for req in server.incoming_requests() {
        tokio::spawn(handle_request(req));
    }

    Ok(())
}

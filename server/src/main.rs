//! This is the server for TestTracker, which uses a PostgreSQL database to store all the data.

use self::passwords::{add_new_user, validate_user};
use color_eyre::Result;
use test_tracker_shared::{ClientToServerMsg, ServerToClientMsg};
use tiny_http::{Header, Request, Response};
use tracing::{error, info, instrument};
use tracing_unwrap::ResultExt;

pub(crate) mod db;
mod passwords;

/// Create a header that will allow the client to function properly without CORS getting in the way.
fn no_cors_header() -> Header {
    Header {
        field: "Access-Control-Allow-Origin"
            .parse()
            .expect("This &'static str should parse just fine"),
        value: "*"
            .parse()
            .expect("This &'static str should parse just fine"),
    }
}

/// Handle a single HTTP request.
#[instrument(skip_all, fields(addr = ?req.remote_addr()))]
async fn handle_request(mut req: Request) -> Result<()> {
    info!("Received a new request");

    let mut body = String::new();
    req.as_reader().read_to_string(&mut body)?;
    let msg: ClientToServerMsg = ron::from_str(&body)?;

    match msg {
        ClientToServerMsg::Authenticate { username, password } => {
            info!(?username, ?password, "Authenticating");
            let validation_result = validate_user(&username, &password).map_err(|e| e.into());
            info!(?validation_result);

            req.respond(
                Response::from_string(
                    ron::to_string(&ServerToClientMsg::AuthenticationResponse(
                        validation_result,
                    ))
                    .expect("Serializing a ServerToClientMsg should never fail"),
                )
                .with_header(no_cors_header()),
            )?;
        }
        ClientToServerMsg::CreateUser { username, password } => {
            info!(?username, ?password, "Creating new user");
            let add_new_user_result = add_new_user(&username, &password).map_err(|e| e.into());
            info!(?add_new_user_result);

            req.respond(
                Response::from_string(
                    ron::to_string(&ServerToClientMsg::AuthenticationResponse(
                        add_new_user_result,
                    ))
                    .expect("Serializing a ServerToClientMsg should never fail"),
                )
                .with_header(no_cors_header()),
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
async fn main() -> Result<()> {
    setup_global_tracing_subscriber();

    info!(port = env!("PORT"), "Initialising server");

    let server = match tiny_http::Server::https(
        concat!("localhost:", env!("PORT")),
        tiny_http::SslConfig {
            certificate: include_bytes!(env!("SERVER_SSL_CERT_PATH")).to_vec(),
            private_key: include_bytes!(env!("SERVER_SSL_KEY_PATH")).to_vec(),
        },
    ) {
        Ok(server) => server,
        Err(error) => {
            error!(
                ?error,
                "Error creating HTTPS server; defaulting to HTTP server"
            );
            tiny_http::Server::http(concat!("localhost:", env!("PORT")))
                .expect_or_log("Unable to create HTTP server")
        }
    };

    info!("Server initialised");

    for req in server.incoming_requests() {
        tokio::spawn(handle_request(req));
    }

    Ok(())
}

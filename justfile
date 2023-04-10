set dotenv-load := true

alias b := build
alias br := build-release
alias sc := serve-client
alias scr := serve-client-release
alias ws := watch-server
alias wsr := watch-server-release

_default:
	@just --list

# build the client and server with optional release flag
_build release='':
	@cd client; touch sass.css sass.css.map _main.css
	cd client; trunk build {{release}}
	cd server; cargo build {{release}}

# build the client and server
build: _build

# build the client and server in release mode
build-release: (_build "--release")

# serve and open the client with Trunk
serve-client:
	cd client; trunk serve --open

# serve and open the client with Trunk in release mode
serve-client-release:
	cd client; trunk serve --release --open

# run the server with cargo watch
watch-server:
	cd server; cargo watch -x run

# run the server in release mode with cargo watch
watch-server-release:
	cd server; cargo watch -x "run --release"

clippy:
	cd client; cargo clippy -- -D clippy::missing_docs_in_private_items -D clippy::semicolon-if-nothing-returned -D clippy::unwrap_used
	cd server; cargo clippy -- -D clippy::missing_docs_in_private_items -D clippy::semicolon-if-nothing-returned -D clippy::unwrap_used
	cd shared; cargo clippy -- -D clippy::missing_docs_in_private_items -D clippy::semicolon-if-nothing-returned -D clippy::unwrap_used

# setup PostgreSQL for test-tracker (assumes a Debian-based OS)
setup:
	sudo apt update
	sudo apt install -y postgresql postgresql-contrib libpq5 libmysqlclient21 libsqlite3-0
	sudo npm i -g -D postcss postcss-cli
	sudo npm i -g cssnano postcss-preset-env sass
	rustup target add wasm32-unknown-unknown
	cargo install diesel_cli trunk cargo-watch
	sudo systemctl start postgresql.service
	sudo -u postgres dropdb --if-exists test_tracker
	sudo -u postgres dropuser --if-exists test_tracker
	sudo -u postgres psql -c "CREATE USER test_tracker WITH CREATEDB LOGIN PASSWORD '{{env_var("PSQL_PASSWORD")}}';"
	sudo -u postgres createdb --owner="test_tracker" test_tracker
	cd server; diesel migration run

# connect to the DB
connect-db:
	psql $DATABASE_URL

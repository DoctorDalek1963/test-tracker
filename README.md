# TestTracker

This application is designed to provide a web interface to a database of past paper completion
statuses. IT DOES NOT STORE PAST PAPERS THEMSELVES - it only stores data about which papers a
person has done.

The first time you open the website, you create an account, and then you can populate your
collection with a list of past papers. Each paper here has a name, a completion status, and an
optional link to the paper and/or the mark scheme, as well as other data, such as your final mark
for the paper and when you did it.

The completion status of each paper is the most important part. TestTracker is designed to help you
keep track of which papers you've already completed and which ones you're yet to complete.

## Quick start

To set this up for yourself, you'll first need:
1. A Linux computer like a [Raspberry Pi](https://www.raspberrypi.com/) to act as the server.
1. A fixed domain, which you can buy, or get for free from somewhere like
   [No-IP](https://www.noip.com/) (make sure to setup the DUC properly).
1. A web server like [nginx](https://nginx.org/en/) or [Apache](https://httpd.apache.org/) running
   on the Raspberry Pi (make sure to let ports 80 and 443 through the firewall and port-forward
   them if necessary).
1. (Technically optional if you're only using it for a personal account) A valid SSL certificate
   with private key. You could pay for these, or get them for free from [Let's
   Encrypt](https://letsencrypt.org/) and use `certbot` to automate the whole process.

Then to actually compile and run this project:
1. Populate the `.env` file for this repo (see [below](#a-sample-.env-file)).
1. Remember to open the port you chose in the `.env` file in the firewall and port-forward it.
1. Install Rust with [rustup](https://rustup.rs/).
1. Install just (`cargo install just`) and run `just setup`.
1. Run `just build-release`.
1. Put the generated server executable and client `dist/` folder in their proper places for your
   web server, and make sure to run the `test-tracker-server` (probably make a systemd service for
   it, or equivalent, so that it runs at startup)

### A sample `.env` file

You `.env` file should look something like this:
```bash
PSQL_PASSWORD=v3rY_53cUR3_pA55w0rD
DATABASE_URL=postgres://test_tracker:${PSQL_PASSWORD}@localhost/test_tracker

SERVER_LOG_PATH=/path/to/server/log/folder
PORT=20519
SERVER_URL=https://myawesomewebsite.com:${PORT}

SERVER_SSL_CERT_PATH=/path/to/ssl/cert.pem
SERVER_SSL_KEY_PATH=/path/to/ssl/privkey.pem
```

If you're doing development, then you will want to prefix every line with `export` so that you can
source the file in your shell.

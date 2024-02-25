//! abanos
//!
//! The command line interface for the abanos programming language.
//!
//! Usage: abanos [OPTIONS]
//!
//! Options:\
//! -H, --host <HOST>  Optional host to connect to [default: 127.0.0.1]\
//! -p, --port <PORT>  Optional port to connect to [default: 8080]\
//! -d, --debug        Optional verbosity level\
//! -h, --help         Print help\
//! -V, --version      Print version\
//!

#[macro_use]
extern crate simple_log;

#[doc(hidden)]
mod connection;
#[doc(hidden)]
mod parse;

use clap::Parser;

/// Mode
///
/// The CLI tool can run in two modes: repl and serialize.
/// In repl mode, the tool will 
/// 1. read expressions from stdin,
/// 1. send them to the server for evaluation, and
/// 1. print the result.
/// In serialize mode, the tool will read expressions from stdin,
/// serialize them as JSON, and output the JSON to stdout.
#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Repl,
    Serialize,
}

// Command line arguments
#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional verbosity level
    #[arg(short, long)]
    debug: bool,

    /// Optional host to connect to
    #[arg(short = 'H', long, default_value = "api.abanos.io")]
    host: String,

    /// Optional mode
    #[arg(short, long, value_enum, default_value = "repl")]
    mode: Mode,

    #[arg(long, default_value = "false")]
    no_tls: bool,

    /// Optional port to connect to
    #[arg(short, long, default_value_t = 443)]
    port: u16,
}

/// Main entry point for repl mode
///
/// This is the main entry point for the CLI. It will parse the command line arguments,
/// create a connection to the server, check its health, and then start a loop to read
/// expressions from stdin, send them to the server for evaluation, and print the result.
///

fn repl(args: Args, token: String) -> Result<(), String> {
    println!("abanos cli v{}", env!("CARGO_PKG_VERSION"));
    println!("copyright (c) 2024 Omar Shorbaji. All rights reserved.");

    debug!("args: {args:?}");

    connection::Connection::new(args.host, args.port, args.no_tls) // create a connection
        .healthcheck(token.clone()) // check its health
        .map(|conn| {
            // if it is healthy
            parse::Parser::new(std::io::stdin().lock()) // repl
                .filter_map(Result::ok)
                .map(|expr| conn.send(expr, token.clone()))
                .for_each(|r| {
                    println!(
                        "{}",
                        match r {
                            Ok(v) => format!("{}", v),
                            Err(e) => format!("{:?}", e),
                        }
                    )
                })
        })
}

/// Main entry point for serialize mode
fn serialize(_args: Args) -> Result<(), String> {
    // need to send prompts to stderr or not prompt at all
    Ok(parse::Parser::new(std::io::stdin().lock()) // repl
        .filter_map(Result::ok)
        .map(|expr| serde_json::to_string(&expr))
        .filter_map(Result::ok)
        .for_each(|r| println!("{}", r)))
}

use oauth2::{basic::BasicClient, revocation::StandardRevocableToken, TokenResponse};
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RevocationUrl, Scope, TokenUrl,
};

use std::env;

fn get_token() -> String {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );

    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(google_client_id, Some(google_client_secret), auth_url, Some(token_url))
        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        .set_redirect_uri(
            RedirectUrl::new("https://api.staging.abanos.io/iam/auth/callback".to_string()).expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_uri(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        );

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!("Open this URL in your browser:\n{authorize_url}\n");

    let mut code = String::new();

    std::io::stdin().read_line(&mut code).unwrap();

    let code = AuthorizationCode::new(code);
    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request(http_client);

    println!(
        "Google returned the following token:\n{:?}\n",
        token_response
    );
    token_response.unwrap().access_token().secret().to_string()
}
#[doc(hidden)]
fn main() -> Result<(), String> {

    // Parse command line arguments
    let args = Args::parse();

    // Set the log level depending on --debug command line argument
    simple_log::quick!(if args.debug { "debug" } else { "info" });

    let token = get_token();
    
    // Run the CLI tool in the mode based on the mode command line argument
    match args.mode {
        Mode::Repl => repl(args, token),
        Mode::Serialize => serialize(args),
    }
}

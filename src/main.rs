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
#[doc(hidden)]
mod token;

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
        .healthcheck() // check its health
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

#[doc(hidden)]
fn main() -> Result<(), String> {
    // Parse command line arguments
    let args = Args::parse();

    // Set the log level depending on --debug command line argument
    simple_log::quick!(if args.debug { "debug" } else { "info" });

    let token = token::get_token(args.host.clone())?;

    // Run the CLI tool in the mode based on the mode command line argument
    match args.mode {
        Mode::Repl => repl(args, token),
        Mode::Serialize => serialize(args),
    }
}

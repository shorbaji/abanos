//! abanos cli
//!
//! Command line interface for the abanos programming language.
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

use clap::{Parser, ValueEnum};

/// Mode
#[derive(ValueEnum, Clone, Debug)]
enum Mode {
    Repl,
    Serialize,
}

// Command line arguments
#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional host to connect to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Optional port to connect to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Optional verbosity level
    #[arg(short, long)]
    debug: bool,

    /// Optional mode
    #[arg(short, long, value_enum, default_value = "repl")]
    mode: Mode,
}

/// Main entry point for repl mode
///
/// This is the main entry point for the CLI. It will parse the command line arguments,
/// create a connection to the server, check its health, and then start a loop to read
/// expressions from stdin, send them to the server for evaluation, and print the result.
///

fn repl(args: Args) -> Result<(), String> {
    println!("abanos cli v{}", env!("CARGO_PKG_VERSION"));
    println!("copyright (c) 2024 Omar Shorbaji. All rights reserved.");

    debug!("args: {args:?}");

    connection::Connection::new(args.host, args.port) // create a connection
        .healthcheck() // check its health
        .map(|conn| {
            // if it is healthy
            parse::Parser::new(std::io::stdin().lock()) // repl
                .filter_map(Result::ok)
                .map(|expr| conn.send(expr))
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
    let args = Args::parse();

    simple_log::quick!(if args.debug { "debug" } else { "info" });

    match args.mode {
        Mode::Repl => repl(args),
        Mode::Serialize => serialize(args),
    }
}

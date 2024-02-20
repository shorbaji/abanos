# `abanos`

> the cloud-native programming language

## Features

- `hosted runtime` runtimes run in the Cloud at abanos.io
- `first-class identity` the programming language is multi-tenant 
- `values as a service` variables, functions and any function is a service
- `access management` software sharing
- `libraries as a service` libraries are services 

## Installation

```sh
git clone https://github.com/shorbaji/abanos.git
cargo build --release
```

## Usage

```sh
abanos [OPTIONS]

Options:
  -H, --host <HOST>  Optional host to connect to [default: 127.0.0.1]
  -p, --port <PORT>  Optional port to connect to [default: 8080]
  -d, --debug        Optional verbosity level
  -m, --mode <MODE>  Optional mode [default: repl] [possible values: repl, serialize]
  -h, --help         Print help
  -V, --version      Print version
```


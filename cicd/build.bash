#!/usr/bin/env bash

set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

printf "Installing cross\n"
sleep 1
cargo install cross --git https://github.com/cross-rs/cross

printf "Building abanos\n"
printf "x86_64-apple-darwin\n"
sleep 1
cargo build --bin abanos --release 
printf "Linux x86_64\n"
cross build --bin abanos --release --target x86_64-unknown-linux-gnu
printf "Linux aarch64\n"
cross build --bin abanos --release --target aarch64-unknown-linux-gnu
printf "Windows x86_64\n"
cross build --bin abanos --release --target x86_64-pc-windows-gnu

mkdir -p ./target/www
cp ./target/release/abanos ./target/www/abanos_x86_64-apple-darwin
cp ./target/x86_64-unknown-linux-gnu/release/abanos ./target/www/abanos_x86_64-linux
cp ./target/aarch64-unknown-linux-gnu/release/abanos ./target/www/abanos_aarch64-linux
cp ./target/x86_64-pc-windows-gnu/release/abanos.exe ./target/www/abanos.exe


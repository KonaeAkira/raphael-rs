#!/usr/bin/env bash

set -euxo pipefail

# install the latest Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# add cargo install dir to PATH
. "$HOME/.cargo/env"

# install wasm32 target
rustup target add wasm32-unknown-unknown

# install Trunk
cargo install --locked trunk

./build-web.sh

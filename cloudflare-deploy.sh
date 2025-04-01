#!/usr/bin/env bash
# This script is run by Cloudflare's worker to build the deployment

set -euxo pipefail

# Base URL used to fetch assets
if [ "$CF_PAGES_BRANCH" == "main" ]; then
    export BASE_URL="https://www.raphael-xiv.com"
else
    export BASE_URL=$CF_PAGES_URL
fi

# Install the Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

rustup update nightly && rustup default nightly
rustup component add rust-src
rustup target add wasm32-unknown-unknown

mv ./.cargo/config.toml ./.cargo/config.toml.backup
mv ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv ./.cargo/config.toml ./.cargo/config_wasm.toml && mv ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

cargo install --locked trunk

# web_sys unstable APIs needed for copy to clipboard functionality
export RUSTFLAGS="--cfg=web_sys_unstable_apis -Ctarget-feature=+atomics,+bulk-memory -Clink-arg=--max-memory=4294967296"

trunk build index.html --release

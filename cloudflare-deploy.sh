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

cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

# install cargo-binstall
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
# install trunk
cargo-binstall --no-confirm --locked trunk

trunk build index.html --release

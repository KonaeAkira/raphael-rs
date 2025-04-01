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
rustup target add wasm32-unknown-unknown

cargo install --locked trunk

# web_sys unstable APIs needed for copy to clipboard functionality
export RUSTFLAGS="--cfg=web_sys_unstable_apis -Ctarget-feature=+atomics,+bulk-memory -Clink-arg=--max-memory=4294967296"

trunk build index.html --release

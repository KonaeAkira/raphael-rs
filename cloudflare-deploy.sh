#!/usr/bin/env bash
# This script is run by Cloudflare's worker to build the deployment

set -euxo pipefail

# Random suffix to invalidate caches for binaries
export RANDOM_SUFFIX="-$(echo $RANDOM$RANDOM | md5sum | head -c 8)"

# Base URL used to fetch assets
if [ "$CF_PAGES_BRANCH" == "main" ]; then
    export BASE_URL="https://www.raphael-xiv.com"
else
    export BASE_URL=$CF_PAGES_URL
fi

# Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
cargo install --locked trunk
rustup target add wasm32-unknown-unknown

# web_sys unstable APIs needed for copy to clipboard functionality
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

trunk build index.html --release

mv distrib/webworker.js distrib/webworker${RANDOM_SUFFIX}.js
mv distrib/webworker_bg.wasm distrib/webworker${RANDOM_SUFFIX}_bg.wasm

#!/usr/bin/env bash

set -exo pipefail

export RUSTFLAGS="--cfg=web_sys_unstable_apis -Ctarget-feature=+atomics,+bulk-memory -Clink-arg=--max-memory=4294967296"
export BASE_URL="http://localhost:8080"

# Requires nightly toolchain, rust-src component, and wasm32-unknown-unkown target
# rustup update nightly && rustup default nightly
# rustup component add rust-src
# rustup target add wasm32-unknown-unknown

cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

trunk serve index.html --release

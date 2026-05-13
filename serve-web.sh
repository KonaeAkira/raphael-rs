#!/usr/bin/env bash

# set -exo pipefail

export BASE_URL="http://localhost:8080"

# Load shared Rust toolchain version (scoped to this script — does not change rustup default)
. ./rust-versions.env

# Makes trunk use the right toolchain version without changing the global default toolchain.
export RUSTUP_TOOLCHAIN="$TOOLCHAIN"

rustup toolchain install "$TOOLCHAIN" --no-self-update
rustup component add --toolchain "$TOOLCHAIN" rust-src
rustup target add --toolchain "$TOOLCHAIN" wasm32-unknown-unknown

cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

trunk serve index.html --release --features=dev-panel

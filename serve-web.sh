#!/usr/bin/env bash

# set -exo pipefail

export BASE_URL="http://localhost:8080"

cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

trunk serve index.html --release --features=dev-panel

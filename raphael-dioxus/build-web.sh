#!/usr/bin/env bash
# Builds the web version of the Dioxus app into `raphael-dioxus/distrib`.

set -euo pipefail
cd "$(dirname "$0")/.."

cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

trunk build --config raphael-dioxus/Trunk.toml --release "$@"

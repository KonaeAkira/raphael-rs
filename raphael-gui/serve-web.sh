#!/usr/bin/env bash
# Serves the web version of the Dioxus app with multi-threading enabled.
# Usage: ./raphael-gui/serve-web.sh [trunk serve args]

set -euo pipefail
cd "$(dirname "$0")/.."

# The wasm config enables atomics & shared memory, which the multi-threaded solver requires.
cp --no-target-directory ./.cargo/config.toml ./.cargo/config.toml.backup
cp --no-target-directory ./.cargo/config_wasm.toml ./.cargo/config.toml
trap "mv --no-target-directory ./.cargo/config.toml.backup ./.cargo/config.toml" EXIT

trunk serve --config raphael-gui/Trunk.toml --release "$@"

#!/usr/bin/env bash

set -exo pipefail

RAPHAEL_TRANSLATIONS_RESET_APPEARANCES=true \
cargo build --package raphael-gui --features=raphael-translations/update-toml

cargo build --package raphael-gui --features=raphael-translations/update-toml --config ./.cargo/config_wasm.toml

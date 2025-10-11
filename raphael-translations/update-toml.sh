#!/usr/bin/env bash

set -exo pipefail

RAPHAEL_TRANSLATIONS_RESET_APPEARANCES=true \
cargo build --package raphael-xiv --features=raphael-translations/update-toml

RUSTFLAGS='-Ctarget-feature=+atomics,+bulk-memory' BASE_URL='' \
cargo +nightly build --package raphael-xiv --features=raphael-translations/update-toml --target wasm32-unknown-unknown

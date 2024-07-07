#!/usr/bin/env bash

set -euxo pipefail

export RANDOM_SUFFIX="-$(hexdump -vn8 -e'2/4 "%08x"' /dev/urandom)"
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

trunk build index.html --public-url="https://www.raphael-xiv.com" --release --dist docs

mv docs/webworker.js docs/webworker${RANDOM_SUFFIX}.js
mv docs/webworker_bg.wasm docs/webworker${RANDOM_SUFFIX}_bg.wasm

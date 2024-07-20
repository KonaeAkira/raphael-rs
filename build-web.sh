#!/usr/bin/env bash

set -euxo pipefail

export RANDOM_SUFFIX="-$(echo $RANDOM$RANDOM | md5sum | head -c 8)"
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

if [ ! -v BASE_URL ]; then
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    export BASE_URL="https://${BRANCH//./-}.raphael-rs.pages.dev"
fi

trunk build index.html --release --dist distrib

mv distrib/webworker.js distrib/webworker${RANDOM_SUFFIX}.js
mv distrib/webworker_bg.wasm distrib/webworker${RANDOM_SUFFIX}_bg.wasm

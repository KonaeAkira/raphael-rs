#!/usr/bin/env bash

set -euxo pipefail

export RANDOM_SUFFIX="-$(echo $RANDOM$RANDOM | md5sum | head -c 8)"
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

if [[ -z "${BASE_URL}" ]]; then
    BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)
    BASE_URL="https://${BRANCH_NAME//./-}.raphael-rs.pages.dev"
    export BASE_URL
fi

trunk build index.html --release --dist distrib

mv distrib/webworker.js distrib/webworker${RANDOM_SUFFIX}.js
mv distrib/webworker_bg.wasm distrib/webworker${RANDOM_SUFFIX}_bg.wasm

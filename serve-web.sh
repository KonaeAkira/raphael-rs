#!/usr/bin/env bash

set -exo pipefail

export RANDOM_SUFFIX=""
# export RUSTFLAGS="--cfg=web_sys_unstable_apis"
export BASE_URL="http://localhost:8080"

trunk serve index.html --dist distrib $1

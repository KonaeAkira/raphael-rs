#!/usr/bin/env bash

set -euxo pipefail

export RANDOM_SUFFIX=""
# export RUSTFLAGS="--cfg=web_sys_unstable_apis"

trunk serve index.html --dist docs

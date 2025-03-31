#!/usr/bin/env bash

set -exo pipefail

export RUSTFLAGS="--cfg=web_sys_unstable_apis -Ctarget-feature=+atomics,+bulk-memory"
export BASE_URL="http://localhost:8080"

trunk serve index.html $1

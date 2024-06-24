#!/usr/bin/env bash

function restore_cname {
    git checkout docs/CNAME
}

# trunk nukes the docs/ directory when building so we need to restore the CNAME record when the script exits
trap restore_cname EXIT

RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk build index.html --public-url="https://www.raphael-xiv.com" --release --dist docs

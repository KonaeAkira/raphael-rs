#!/usr/bin/env bash
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk build gui/index.html --public-url="https://www.raphael-xiv.com" --release --dist docs

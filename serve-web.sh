#!/usr/bin/env bash
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve gui/index.html --dist docs

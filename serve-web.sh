#!/usr/bin/env bash

RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve index.html --dist docs $1

#!/bin/bash
cargo watch -x 'test --all -- --nocapture' -x build -x 'fmt --all' -x 'clippy --all-targets --all-features' -x 'run -- serve'

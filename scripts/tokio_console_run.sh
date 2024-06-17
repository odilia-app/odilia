#!/bin/sh
RUSTFLAGS="--cfg tokio_unstable" RUST_LOG="trace" cargo run

#!/usr/bin/env bash

export RUST_BACKTRACE=1
cargo test --features cli_tests && \
  {
    echo "FINISHED DEBUG TESTING SUCCESSFULLY, RUNNING RELEASE TESTS:"
    echo "==========================================================="
    cargo test --release --features cli_tests
  }


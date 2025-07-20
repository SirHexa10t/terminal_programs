#!/usr/bin/env bash

# opt-level: 3: maximum speed   s: optimize for size   z: even smaller

time RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release && \
  echo "SUCCESSFULLY BUILT. OUTPUT:"
  ls -lh --color=always --time-style="+%F_%T" target/release/

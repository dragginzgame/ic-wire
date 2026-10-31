#!/usr/bin/env bash
set -euo pipefail
out="${1:?prepared study directory}"
export CARGO_TARGET_DIR="$out/compression-target"
mkdir -p "$out/compression-results"
for profile in evaluation benchmark-speed; do
 for lane in copy lz4; do
  features=wire
  [[ "$lane" != lz4 ]] || features=wire,compression
  cargo build --manifest-path "$out/compression/Cargo.toml" --locked --offline -p ic-wire-benchmarks --example compression-wasm --target wasm32-unknown-unknown --profile "$profile" --features "$features"
  cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/compression_wasm.wasm" "$out/compression-results/$profile-$lane.wasm"
 done
done
cargo run --manifest-path "$out/compression/Cargo.toml" --locked --offline -p ic-wire-benchmarks --example compression-probe --features wire,ic -- "$out/compression-results"

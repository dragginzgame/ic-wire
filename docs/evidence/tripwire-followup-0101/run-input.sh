#!/usr/bin/env bash
set -euo pipefail
out="${1:?prepared study directory}"
src="$out/input"
export CARGO_TARGET_DIR="$out/input-target"
for profile in evaluation benchmark-speed; do
 dir="$out/input-$profile"
 mkdir -p "$dir"
 for lane in wire_owned wire_borrowed wire_direct wire_selective; do
  case "$lane" in wire_owned) features=wire;; wire_borrowed) features=borrowed-input;; wire_direct) features=direct-owned-input;; wire_selective) features=selective-input;; esac
  cargo build --manifest-path "$src/Cargo.toml" --locked --offline --profile "$profile" -p ic-wire-benchmarks --example selective-input --target wasm32-unknown-unknown --features "$features"
  cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/selective_input.wasm" "$dir/$lane.wasm"
 done
 SELECTIVE_EVIDENCE="$dir" cargo test --manifest-path "$src/Cargo.toml" --locked --offline -p ic-wire-benchmarks --example selective-input --features ic physical_updates -- --ignored --test-threads=1
 done

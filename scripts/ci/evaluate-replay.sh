#!/usr/bin/env bash
set -euo pipefail
# Local correctness trial, excluded from matched codec size/performance builds.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
export CARGO_TARGET_DIR="$root/target"
output="$root/target/replay-evaluation-evidence"
mkdir -p "$output"
cargo build --locked -p ic-wire-canister-evaluation --lib --features replay --target wasm32-unknown-unknown --profile canister-evaluation
cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/canister-evaluation/ic_wire_canister_evaluation.wasm" "$output/replay.wasm"
cargo run --quiet --locked -p ic-wire-canister-evaluation --features replay --example replay-fixture -- "$output/replay.wasm" "$output/report.pending.json"
mv "$output/report.pending.json" "$output/report.json"
printf 'Verified replay report: %s/report.json\n' "$output"

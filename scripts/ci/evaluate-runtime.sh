#!/usr/bin/env bash
set -euo pipefail
# Local installed runtime comparison; uses only a prepared PocketIC server.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
export CARGO_TARGET_DIR="$root/target"
output="$root/target/runtime-comparison"
mkdir -p "$output"
rustc -vV > "$output/build-rustc.txt"
printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\n' "${RUSTFLAGS:-}" "${CARGO_ENCODED_RUSTFLAGS:-}" > "$output/build-flags.txt"
for mode in internal management; do
    for codec in candid wire; do
        features=runtime-benchmark
        [[ "$codec" != candid ]] || features="$features,candid-codec"
        [[ "$mode" != management ]] || features="$features,management"
        cargo build --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --profile canister-evaluation --features "$features"
        cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/canister-evaluation/ic_wire_canister_evaluation.wasm" "$output/$codec-$mode.wasm"
    done
done
cargo run --quiet --locked -p ic-wire-canister-evaluation --features runtime-benchmark --example runtime-measure -- "$output"
mv "$output/report.pending.json" "$output/report.json"
printf 'Verified runtime comparison: %s/report.json\n' "$output"

#!/usr/bin/env bash
set -euo pipefail
# Explicit local experiment; never publishes or contacts the live IC.
# Requires fetched locked dependencies, the Wasm target and a prepared PocketIC.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
export CARGO_TARGET_DIR="$root/target"
output="$root/target/canister-evaluation-evidence"
mkdir -p "$output"
rustc -vV > "$output/build-rustc.txt"
printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\n' "${RUSTFLAGS:-}" "${CARGO_ENCODED_RUSTFLAGS:-}" > "$output/build-flags.txt"
for mode in internal management; do
    for codec in candid postcard; do
        features=''
        [[ "$codec" != candid ]] || features=candid-codec
        [[ "$mode" != management ]] || features="${features:+$features,}management"
        options=()
        if [[ -n "$features" ]]; then
            options=(--features "$features")
        fi
        cargo build --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --profile canister-evaluation ${options[@]+"${options[@]}"}
        cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/canister-evaluation/ic_wire_canister_evaluation.wasm" "$output/$codec-$mode.wasm"
    done
done
cargo run --quiet --locked -p ic-wire-canister-evaluation --example measure-canisters -- "$output"
mv "$output/report.pending.json" "$output/report.json"
printf 'Verified experiment report: %s/report.json\n' "$output"

#!/usr/bin/env bash
# Identical inputs, lock, compiler, target and profiles; change hook ownership only.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
[[ $# == 1 ]] || { echo 'usage: evaluate-endpoint.sh NEW_OUTPUT_DIRECTORY' >&2; exit 2; }
: "${POCKET_IC_BIN:?prepare and select a PocketIC server explicitly}"
mkdir -p "$(dirname "$1")"
mkdir "$1"
out="$(cd "$1" && pwd -P)"
rustc -Vv > "$out/compiler.txt"
uname -a > "$out/host.txt"
cp Cargo.lock "$out/Cargo.lock"
shasum -a 256 Cargo.lock Cargo.toml crates/ic-wire/Cargo.toml crates/ic-wire-derive/Cargo.toml crates/ic-wire-derive/src/endpoint/mod.rs crates/ic-wire-endpoint/Cargo.toml crates/ic-wire-endpoint/src/lib.rs examples/endpoint/Cargo.toml examples/endpoint/examples/cost/mod.rs "$POCKET_IC_BIN" > "$out/inputs.sha256"
printf 'profile,lane,total_wasm_bytes,code_section_bytes\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    mkdir "$out/$profile"
    for lane in inline extracted; do
        features=()
        [[ "$lane" != inline ]] || features=(--features inline-baseline)
        cargo build --locked --profile "$profile" -p ic-wire-endpoint-fixture --example endpoint-cost --target wasm32-unknown-unknown "${features[@]}" > "$out/$profile/$lane-build.log" 2>&1
        artifact="${CARGO_TARGET_DIR:-$PWD/target}/wasm32-unknown-unknown/$profile/examples/endpoint_cost.wasm"
        cp "$artifact" "$out/$profile/$lane.wasm"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        code="$(cargo run --quiet --locked -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact")"
        printf '%s,%s,%s,%s\n' "$profile" "$lane" "$bytes" "$code" >> "$out/wasm.csv"
    done
    ENDPOINT_COST_DIR="$out/$profile" cargo test --locked -p ic-wire-endpoint-fixture --example endpoint-cost --features ic -- ic_cost --ignored --nocapture > "$out/$profile/ic.log" 2>&1
    ENDPOINT_NATIVE_REPORT="$out/$profile/native.json" cargo test --locked --profile "$profile" -p ic-wire-endpoint-fixture --example endpoint-cost -- native_cost --ignored --nocapture > "$out/$profile/native.log" 2>&1
    shasum -a 256 "$out/$profile/inline.wasm" "$out/$profile/extracted.wasm" > "$out/$profile/artifacts.sha256"
done

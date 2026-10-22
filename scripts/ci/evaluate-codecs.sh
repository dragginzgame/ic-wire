#!/usr/bin/env bash
# Native/heap/Wasm comparison; optional local PocketIC uses an explicit server.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
out="${1:-$root/target/codec-comparison}"
case "$out" in /*) ;; *) out="$root/$out" ;; esac
mkdir "$out"
mkdir -p "$out/source/scripts/ci"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/source/"
cp -R crates "$out/source/"
cp scripts/ci/evaluate-codecs.sh "$out/source/scripts/ci/"
runs="${RUNS:-2}"
samples="${SAMPLES:-11}"
batch_ms="${BATCH_MS:-2}"
case "$runs" in ''|*[!0-9]*) echo 'RUNS must be a positive integer' >&2; exit 1;; esac
if [ "$runs" -lt 1 ]; then echo 'RUNS must be positive' >&2; exit 1; fi
printf 'profile,codec,wasm_bytes,gzip_bytes,wasm_sha256\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    run=1
    while [ "$run" -le "$runs" ]; do
        cargo bench --locked --profile "$profile" -p ic-wire-benchmarks --bench suite --features wire,candid -- \
            "$out/$profile-native-$run" "$profile" "$samples" "$batch_ms" > "$out/$profile-native-$run.log" 2>&1
        run=$((run + 1))
    done
    for mode in allocations heap; do
        cargo bench --locked --profile "$profile" -p ic-wire-benchmarks --bench suite --features "wire,candid,$mode" -- \
            "$out/$profile-$mode" "$profile" "$samples" "$batch_ms" > "$out/$profile-$mode.log" 2>&1
    done
    for codec in wire candid; do
        cargo build --locked --profile "$profile" -p ic-wire-benchmarks --example codec-wasm \
            --target wasm32-unknown-unknown --features "$codec" --message-format=json \
            > "$out/$profile-$codec-build.jsonl" 2> "$out/$profile-$codec-wasm.log"
        artifact="$(cargo run --locked -p ic-wire-benchmarks --example codec-report -- \
            --artifact "$out/$profile-$codec-build.jsonl" 2>> "$out/$profile-$codec-wasm.log")"
        cp "$artifact" "$out/$profile-$codec.wasm"
        gzip -n -9 -c "$artifact" > "$out/$profile-$codec.wasm.gz"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        compressed="$(wc -c < "$out/$profile-$codec.wasm.gz" | tr -d ' ')"
        digest="$(shasum -a 256 "$artifact")"
        printf '%s,%s,%s,%s,%s\n' "$profile" "$codec" "$bytes" "$compressed" "${digest%% *}" >> "$out/wasm.csv"
    done
done
cargo run --locked -p ic-wire-benchmarks --example codec-report -- "$out"
if [ -n "${POCKET_IC_BIN:-}" ]; then
    cargo run --locked -p ic-wire-benchmarks --example codec-instructions --features wire,candid,ic -- "$out" > "$out/instructions.log" 2>&1
fi
printf 'Evidence: %s\n' "$out"

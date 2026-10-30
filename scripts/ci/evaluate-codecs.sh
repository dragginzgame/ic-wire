#!/usr/bin/env bash
# Native/heap/Wasm comparison; optional local PocketIC uses an explicit server.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
out="${1:-$root/target/codec-comparison}"
case "$out" in /*) ;; *) out="$root/$out" ;; esac
corpus="${CORPUS_FEATURE:-}"
primary=""
while IFS='|' read -r name features lane; do
    if [ "$corpus" = "$features" ]; then primary="$lane"; break; fi
done < scripts/ci/codec-cohorts.psv
[ -n "$primary" ] || { echo "unknown corpus: $corpus" >&2; exit 1; }
extra="${corpus:+,$corpus}"
mkdir "$out"
if [ -n "${CODEC_SOURCE_SNAPSHOT:-}" ]; then
    # Matrix cohorts execute from one frozen source tree. Retain its location
    # rather than copying that same tree for every cohort.
    [ "$CODEC_SOURCE_SNAPSHOT" = "$root" ] || { echo 'snapshot must be the executing source tree' >&2; exit 1; }
    [ "$(cd "$out/.." && pwd)/source" = "$root" ] || { echo 'snapshot must be adjacent to cohort output' >&2; exit 1; }
    printf '../source\n' > "$out/source-location.txt"
else
    mkdir -p "$out/source/scripts/ci"
    cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/source/"
    cp -R crates examples "$out/source/"
    cp scripts/ci/evaluate-codecs.sh scripts/ci/codec-cohorts.psv "$out/source/scripts/ci/"
fi
runs="${RUNS:-2}"
samples="${SAMPLES:-11}"
batch_ms="${BATCH_MS:-2}"
case "$runs" in ''|*[!0-9]*) echo 'RUNS must be a positive integer' >&2; exit 1;; esac
if [ "$runs" -lt 1 ]; then echo 'RUNS must be positive' >&2; exit 1; fi
printf 'profile,codec,elapsed_seconds\n' > "$out/build-times.csv"
printf 'profile,codec,wasm_bytes,wasm_sha256,code_section_bytes\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    run=1
    while [ "$run" -le "$runs" ]; do
        cargo bench --locked --profile "$profile" -p ic-wire-benchmarks --bench suite --features "wire,candid$extra" -- \
            "$out/$profile-native-$run" "$profile" "$samples" "$batch_ms" > "$out/$profile-native-$run.log" 2>&1
        run=$((run + 1))
    done
    for mode in allocations heap; do
        cargo bench --locked --profile "$profile" -p ic-wire-benchmarks --bench suite --features "wire,candid,$mode$extra" -- \
            "$out/$profile-$mode" "$profile" "$samples" "$batch_ms" > "$out/$profile-$mode.log" 2>&1
    done
    for codec in wire candid; do
        started="$SECONDS"
        cargo build --locked --profile "$profile" -p ic-wire-benchmarks --example codec-wasm \
            --target wasm32-unknown-unknown --features "$codec$extra" --message-format=json \
            > "$out/$profile-$codec-build.jsonl" 2> "$out/$profile-$codec-wasm.log"
        printf '%s,%s,%s\n' "$profile" "$codec" "$((SECONDS - started))" >> "$out/build-times.csv"
        artifact="$(cargo run --locked -p ic-wire-benchmarks --example codec-report -- \
            --artifact "$out/$profile-$codec-build.jsonl" 2>> "$out/$profile-$codec-wasm.log")"
        cp "$artifact" "$out/$profile-$codec.wasm"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        digest="$(shasum -a 256 "$artifact")"
        code_bytes="$(cargo run --locked -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact" 2>> "$out/$profile-$codec-wasm.log")"
        printf '%s,%s,%s,%s,%s\n' "$profile" "$codec" "$bytes" "${digest%% *}" "$code_bytes" >> "$out/wasm.csv"
    done
done
cargo run --locked -p ic-wire-benchmarks --example codec-report -- "$out" "$primary"
if [ -n "${POCKET_IC_BIN:-}" ]; then
    cargo run --locked -p ic-wire-benchmarks --example codec-instructions --features "wire,candid,ic$extra" -- "$out" > "$out/instructions.log" 2>&1
fi
printf 'Evidence: %s\n' "$out"

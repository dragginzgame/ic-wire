#!/usr/bin/env bash
set -euo pipefail

# Requires an explicitly prepared nightly toolchain, C++ compiler and Cargo cache.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
export RUSTUP_AUTO_INSTALL=0
toolchain=nightly-2026-07-29
runs="${FUZZ_RUNS:-100000}"
[[ "$runs" =~ ^[1-9][0-9]*$ ]] || { printf 'FUZZ_RUNS must be a positive integer\n' >&2; exit 1; }
host="$(rustup run "$toolchain" rustc -vV | sed -n 's/^host: //p')"
[[ -n "$host" ]] || { printf 'Missing Rust host identity\n' >&2; exit 1; }
export CARGO_TARGET_DIR="$root/target/fuzz-runtime"
mkdir -p "$CARGO_TARGET_DIR/corpus" "$CARGO_TARGET_DIR/artifacts"
perl scripts/ci/seed-packed-fuzz.pl "$CARGO_TARGET_DIR/corpus"
# An explicit target keeps these flags off proc-macro/build-script host binaries.
export RUSTFLAGS='-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Zsanitizer=address'
rustup run "$toolchain" cargo run --offline --locked --release --target "$host" \
    -p ic-wire --example fuzz-runtime -- \
    "$CARGO_TARGET_DIR/corpus" "-artifact_prefix=$CARGO_TARGET_DIR/artifacts/" \
    "-runs=$runs" -seed=42 -max_len=2048 -timeout=5 -rss_limit_mb=2048

#!/usr/bin/env bash
# Frozen production entrypoints, all exports, complete physical execution costs.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?select a prepared instrumented PocketIC server}"
out="${1:?fresh absolute evidence directory}"
case "$out" in /*) ;; *) echo 'output must be absolute' >&2; exit 1;; esac
mkdir "$out"
mkdir "$out/source"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/rust-toolchain.toml" "$out/source/"
cp -R "$root/crates" "$root/examples" "$out/source/"
cp "$0" "$out/runner.sh"
cp "$root/scripts/ci/summarize-endpoint-matrix.mjs" "$out/summarize.mjs"
git -C "$root" rev-parse HEAD > "$out/base-commit.txt"
rustc -vV > "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" "$out/source/Cargo.lock" > "$out/hashes.txt"
find "$out/source" -type f -print0 | sort -z | xargs -0 sha256sum > "$out/source-hashes.txt"
export CARGO_TARGET_DIR="$out/target"
cd "$out/source"
for profile in evaluation benchmark-speed; do
    directory="$out/$profile"
    mkdir "$directory"
    cargo build --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile" > "$directory/build.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$directory/endpoint.wasm"
    node "$out/summarize.mjs" "$directory" --exports
    cargo run --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$directory/endpoint.wasm" > "$directory/code-bytes.txt" 2> "$directory/report-build.log"
    ENDPOINT_MATRIX_DIR="$directory" cargo test --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$directory/ic.log" 2>&1
    node "$out/summarize.mjs" "$directory" > "$directory/summary.log"
done

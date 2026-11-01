#!/usr/bin/env bash
set -euo pipefail
candidate=/tmp/ic-wire-pr41-registry-20261101
study="$candidate/target/runtime-registry-costs"
mkdir -p "$study"
export CARGO_TARGET_DIR="$study/target"
: "${POCKET_IC_BIN:?prepared instrumented PocketIC}"
rustc -vV > "$study/compiler.txt"
sha256sum "$POCKET_IC_BIN" > "$study/server-before.sha256"
for lane in platform016 runtime0171; do
    if [[ "$lane" == platform016 ]]; then source=/tmp/ic-wire-pr41-baseline-20261101; else source="$candidate"; fi
    cd "$source"
    cp Cargo.lock "$study/$lane.lock"
    for profile in evaluation benchmark-speed; do
        directory="$study/costs/$lane/$profile"
        mkdir -p "$directory"
        cargo build --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile" > "$directory/build.log" 2>&1
        cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$directory/endpoint.wasm"
        node scripts/ci/summarize-endpoint-matrix.mjs "$directory" --exports
        /home/adam/projects/ic-wire/target/debug/examples/codec-report --code-size "$directory/endpoint.wasm" > "$directory/code-bytes.txt"
        ENDPOINT_MATRIX_DIR="$directory" cargo test --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$directory/ic.log" 2>&1
        node scripts/ci/summarize-endpoint-matrix.mjs "$directory" > "$directory/summary.log"
    done
done
sha256sum "$POCKET_IC_BIN" > "$study/server-after.sha256"
cmp "$study/server-before.sha256" "$study/server-after.sha256"

#!/usr/bin/env bash
set -euo pipefail
study=/home/adam/projects/ic-wire/target/runtime-017-adoption
export CARGO_TARGET_DIR="$study/wire/target"
: "${POCKET_IC_BIN:?select the instrumented prepared server}"
rustc -vV > "$study/compiler.txt"
sha256sum "$POCKET_IC_BIN" > "$study/server.sha256"
for lane in platform016 runtime017; do
    if [[ "$lane" == platform016 ]]; then source=/tmp/wire-runtime-owner-baseline-20261101; else source=/tmp/wire-runtime-owner-20261101; fi
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

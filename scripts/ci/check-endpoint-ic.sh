#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
: "${POCKET_IC_BIN:?prepare and select a PocketIC server explicitly}"
cargo build --locked -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile evaluation
export ENDPOINT_WASM="${CARGO_TARGET_DIR:-$PWD/target}/wasm32-unknown-unknown/evaluation/ic_wire_endpoint_fixture.wasm"
cargo test --locked -p ic-wire-endpoint-fixture --test installed --features ic -- --ignored --nocapture

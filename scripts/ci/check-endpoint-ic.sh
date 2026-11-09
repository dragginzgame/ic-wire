#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
: "${POCKET_IC_BIN:?prepare and select a PocketIC server explicitly}"
cargo build --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --lib --features admission-contract --target wasm32-unknown-unknown --profile evaluation
export ENDPOINT_WASM="${CARGO_TARGET_DIR:-$PWD/target}/wasm32-unknown-unknown/evaluation/ic_wire_endpoint_fixture.wasm"
cargo test --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --test endpoints --features ic,admission-contract -- --exact installed::typed_calls_limits_and_transactions --ignored --nocapture

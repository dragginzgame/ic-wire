#!/usr/bin/env bash
# Focused native/compile qualification; installed execution is explicit below.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export RUSTUP_AUTO_INSTALL=0
mkdir -p target
output="$(mktemp -d "$PWD/target/endpoint-check.XXXXXX")"
printf 'Endpoint compile evidence: %s\n' "$output"
cargo test --locked -p ic-wire-endpoint-fixture --lib --features schema
cargo test --locked -p ic-wire --lib automatic_method_identity_vectors_and_frame
cargo clippy --locked -p ic-wire-endpoint -p ic-wire -p ic-wire-derive --lib --features ic-wire-endpoint/typescript -- -D warnings
cargo clippy --locked -p ic-wire-endpoint-fixture --lib --tests --example endpoint-cost --features ic,schema,inline-baseline -- -D warnings
cargo build --locked -p ic-wire-endpoint --message-format=json > "$output/artifacts.jsonl"
cargo run --locked -p ic-wire-qualification --example check-diagnostics -- "$output" --endpoint
cargo build --locked -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile evaluation
cargo +1.91.0 check --locked -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown
cargo tree --locked -p ic-wire --no-default-features --edges normal,build --prefix none > "$output/core-graph.txt"
cargo tree --locked -p ic-wire-endpoint-fixture --target wasm32-unknown-unknown --edges normal,build --prefix none > "$output/endpoint-graph.txt"
# Match package names, not local source paths (for example, a "candidate" copy).
if grep -Eq '^(ic-platform|canic|candid|pocket-ic) v' "$output/core-graph.txt" || grep -Eq '^(canic|candid|pocket-ic|ic-cdk-executor|ic0) v' "$output/endpoint-graph.txt"; then
    echo 'unexpected runtime dependency' >&2
    exit 1
fi

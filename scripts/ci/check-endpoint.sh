#!/usr/bin/env bash
# Focused native/compile qualification; installed execution is explicit below.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export RUSTUP_AUTO_INSTALL=0
node --test scripts/ci/summarize-endpoint-matrix.test.mjs
mkdir -p target
output="$(mktemp -d "$PWD/target/endpoint-check.XXXXXX")"
printf 'Endpoint compile evidence: %s\n' "$output"
bash scripts/ci/check-compatible-consumers.sh
cargo test --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --lib
cargo test --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --lib --features schema
cargo test --locked -p ic-wire --lib automatic_method_identity_vectors_and_frame
# Public-package lint (including schemas) belongs to make lint; this owns the fixture.
cargo clippy --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --lib --tests --features ic,schema -- -D warnings
# Compile the benchmark's actual reverse-consumer path without running a cost study.
cargo test --locked --manifest-path qualification/Cargo.toml -p ic-wire-benchmarks --example input-borrowing --features ic,endpoint-input --no-run
cargo build --locked -p ic-wire-endpoint --message-format=json > "$output/artifacts.jsonl"
cargo run --locked --manifest-path qualification/Cargo.toml -p ic-wire-qualification --example check-diagnostics --features diagnostics -- "$output" --endpoint
cargo build --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile evaluation
cargo tree --locked -p ic-wire --no-default-features --edges normal,build --prefix none > "$output/core-graph.txt"
cargo tree --locked --manifest-path qualification/Cargo.toml -p ic-wire-endpoint-fixture --target wasm32-unknown-unknown --edges normal,build --prefix none > "$output/endpoint-graph.txt"
# Match package names, not local source paths (for example, a "candidate" copy).
# Runtime 0.21.1 owns the transitive ic0 binding; direct consumer edges remain
# forbidden by the Cargo boundary checker, and Core must stay independent.
if grep -Eq '^(ic-wire-endpoint|ic-wire-endpoint-macros|ic-platform|ic-platform-runtime|ic-platform-pocket-ic|ic-testkit|canic|candid|pocket-ic|ic0) v' "$output/core-graph.txt" || grep -Eq '^(ic-platform|ic-platform-pocket-ic|ic-testkit|canic|candid|pocket-ic|ic-cdk-executor) v' "$output/endpoint-graph.txt"; then
    echo 'unexpected runtime dependency' >&2
    exit 1
fi

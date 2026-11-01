#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
out="$PWD/target/testkit-076-adoption"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export POCKET_IC_BIN=/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic
features=ic-wire-benchmarks/wire,ic-wire-benchmarks/candid,ic-wire-benchmarks/ic,ic-wire-benchmarks/cost-smoke,ic-wire-endpoint-fixture/ic,ic-wire-endpoint-fixture/schema
cargo clippy --locked --offline -p ic-wire-benchmarks -p ic-wire-endpoint-fixture --all-targets --features "$features" -- -D warnings > "$out/clippy.log" 2>&1
RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked --offline -p ic-wire-benchmarks -p ic-wire-endpoint-fixture --all-targets --features "$features" > "$out/msrv.log" 2>&1
cargo tree --locked --offline -p ic-wire --no-default-features --edges normal,build --prefix none > "$out/core-graph.txt"
cargo tree --locked --offline -p ic-wire-endpoint-fixture --target wasm32-unknown-unknown --edges normal,build --prefix none --format '{p} features={f}' > "$out/endpoint-wasm-graph.txt"
cargo tree --locked --offline -p ic-wire-benchmarks --features wire,candid,ic,cost-smoke --edges normal,build,dev --prefix none --format '{p} features={f}' > "$out/benchmark-host-graph.txt"
if rg -q '^(ic-testkit|pocket-ic|candid|ic-platform) v' "$out/core-graph.txt"; then echo 'Unexpected Core dependency' >&2; exit 1; fi
if rg -q '^(ic-testkit|pocket-ic|candid|serde_json|ic-cdk-executor) v' "$out/endpoint-wasm-graph.txt"; then echo 'Host dependency leaked into Wasm' >&2; exit 1; fi
if rg -q '^ic-platform v0\.15' "$out/benchmark-host-graph.txt"; then echo 'Old Platform remains in host qualification' >&2; exit 1; fi
rg -q '^ic-testkit v0\.76\.0' "$out/benchmark-host-graph.txt"
rg -q '^ic-platform v0\.16\.0' "$out/benchmark-host-graph.txt"
bash scripts/ci/check-endpoint-ic.sh > "$out/installed.log" 2>&1
echo 'Testkit adoption: caller Clippy, Rust 1.91, host/Wasm isolation and installed endpoint qualification passed.'

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
out="$PWD/target/build-host-0735-adoption"
make release-editor-check > "$out/editor-check.log" 2>&1
make check-typed-diagnostics > "$out/diagnostics.log" 2>&1
cargo clippy --locked --offline -p ic-wire-qualification --example check-diagnostics -- -D warnings > "$out/clippy.log" 2>&1
RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked --offline -p ic-wire-qualification --example check-diagnostics > "$out/msrv.log" 2>&1
cargo tree --locked --offline -p ic-wire --no-default-features --edges normal,build --prefix none > "$out/core-graph.txt"
cargo tree --locked --offline -p ic-wire-endpoint --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/endpoint-wasm-graph.txt"
if rg -q '^ic-(build|host-|testkit)' "$out/core-graph.txt" "$out/endpoint-wasm-graph.txt"; then echo 'Host tooling leaked into production' >&2; exit 1; fi
make check-tooling > "$out/make-fixture.log" 2>&1
make check-release > "$out/release-fixtures.log" 2>&1
echo 'Build/Host adoption: editor, diagnostics, caller lint/MSRV, graph isolation and disposable Make/release fixtures passed.'

#!/usr/bin/env bash
set -euo pipefail
# Compile explicit cases against the exact locked Cargo-built libraries.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
export RUSTUP_AUTO_INSTALL=0
output="$root/target/ui-diagnostics"
mkdir -p "$output"
run="$(mktemp -d "$output/wire.XXXXXX")"
printf 'Diagnostic logs: %s\n' "$run"
status=0
trap 'status=$?; if [[ "$status" != 0 ]]; then printf "Diagnostic failure logs: %s\n" "$run" >&2; fi' EXIT
cargo build --offline --locked -p ic-wire \
    --message-format=json > "$run/artifacts.jsonl"
cargo run --quiet --offline --locked -p ic-wire-canister-evaluation \
    --example check-diagnostics -- "$run"

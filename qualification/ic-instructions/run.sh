#!/usr/bin/env bash
set -euo pipefail
: "${POCKET_IC_BIN:?set POCKET_IC_BIN to a prepared compatible server}"
for variable in RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; do
    if [[ -n "${!variable:-}" ]]; then echo "unset $variable for the fixed comparison" >&2; exit 1; fi
done
if env | rg -q '^CARGO_(PROFILE_.*|TARGET_.*_RUSTFLAGS)='; then
    echo 'unset Cargo profile/target flag overrides' >&2; exit 1
fi
mkdir -p qualification/target
output=$(mktemp -d "$PWD/qualification/target/ic-counter-XXXXXX")
printf 'Instruction evidence: %s\n' "$output"
rustc -Vv > "$output/compiler.txt"
for profile in wire-size wire-speed; do
    for supplier in candidate reference; do
        target="$output/$profile-$supplier"
        cargo build --manifest-path qualification/Cargo.toml -p ic-wire-consumer --no-default-features --features "$supplier,ic" --example instruction-counter --target wasm32-unknown-unknown --profile "$profile" --target-dir "$target" --locked --offline
        cp "$target/wasm32-unknown-unknown/$profile/examples/instruction_counter.wasm" "$output/$profile-$supplier.wasm"
    done
done
cargo run --manifest-path qualification/ic-instructions/Cargo.toml --locked --offline -- "$output"

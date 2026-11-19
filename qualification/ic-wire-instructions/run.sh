#!/usr/bin/env bash
set -euo pipefail
: "${POCKET_IC_BIN:?set POCKET_IC_BIN to a prepared compatible server}"
[[ -f "$POCKET_IC_BIN" && -x "$POCKET_IC_BIN" ]] || { echo 'Prepared executable PocketIC server required' >&2; exit 1; }
unset IC_WIRE_POCKET_IC_URL
for variable in RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; do
    if [[ -n "${!variable:-}" ]]; then echo "unset $variable for the fixed measurement" >&2; exit 1; fi
done
if env | rg -q '^CARGO_(PROFILE_.*|TARGET_.*_RUSTFLAGS)='; then
    echo 'unset Cargo profile/target flag overrides' >&2; exit 1
fi
mkdir -p qualification/target
output=$(mktemp -d "$PWD/qualification/target/ic-counter-XXXXXX")
printf 'Instruction evidence: %s\n' "$output"
rustc -Vv > "$output/compiler.txt"
if [[ ${1:-} == --numeric ]]; then
    for profile in wire-size wire-speed; do
        cargo build --manifest-path qualification/Cargo.toml -p ic-wire-compatibility --locked --profile "$profile" --target wasm32-unknown-unknown --features candid-comparison --example numeric-bounded --example numeric-ordinary --example numeric-packed --example numeric-candid
        for codec in ordinary bounded packed candid; do
            cp "qualification/target/wasm32-unknown-unknown/$profile/examples/numeric-$codec.wasm" "$output/$profile-$codec.wasm"
        done
    done
    cargo test --manifest-path qualification/Cargo.toml -p ic-wire-compatibility --locked --features candid-comparison --example numeric-bounded --example numeric-ordinary --example numeric-packed --example numeric-candid -- --nocapture --test-threads=1 > "$output/allocations.txt"
    exec cargo run --manifest-path qualification/Cargo.toml -p ic-wire-instructions --locked -- "$output" --numeric
fi
for profile in wire-size wire-speed; do
    target="$output/$profile"
    cargo build --manifest-path qualification/Cargo.toml -p ic-wire-consumer --features ic --example instruction-counter --target wasm32-unknown-unknown --profile "$profile" --target-dir "$target" --locked
    cp "$target/wasm32-unknown-unknown/$profile/examples/instruction_counter.wasm" "$output/$profile.wasm"
done
cargo run --manifest-path qualification/Cargo.toml -p ic-wire-instructions --locked -- "$output"

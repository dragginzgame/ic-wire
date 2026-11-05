#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target/schema-targets"
fixture="$(mktemp -d "$root/target/schema-targets/consumer.XXXXXX")"
printf 'Schema forwarding fixture: %s\n' "$fixture"
cp "$root/crates/ic-wire-qualification/tests/fixtures/schema_targets/mod.rs" "$fixture/lib.rs"
# Start with the selected workspace versions; resolution may only update this
# disposable consumer lock. Every build/check after resolution is locked/offline.
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cat > "$fixture/Cargo.toml" <<EOF_MANIFEST
[workspace]
[package]
name = "wire-schema-targets"
version = "0.0.0"
edition = "2024"
rust-version = "$RUSTUP_TOOLCHAIN"
[lib]
path = "lib.rs"
[features]
wire-schema = ["ic-wire/typescript"]
[dependencies]
ic-wire = { path = "$root/crates/ic-wire" }
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
EOF_MANIFEST
export CARGO_TARGET_DIR="$root/target/schema-targets/build"
cargo metadata --offline --format-version 1 --manifest-path "$fixture/Cargo.toml" > "$fixture/metadata.json"
# Standalone Cargo invocations prevent workspace feature unification from
# accidentally hiding a metadata dependency in ordinary codec consumers.
cargo build --locked --offline --manifest-path "$fixture/Cargo.toml" --message-format=json > "$fixture/runtime-artifacts.jsonl"
cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" \
    --edges normal,build --prefix none > "$fixture/host-runtime.txt"
if rg '^ic-wire-derive v' "$fixture/host-runtime.txt" || rg '"name":"ic_wire_derive"' "$fixture/runtime-artifacts.jsonl"; then
    echo 'Codec-only consumer compiled the optional metadata derive' >&2
    exit 1
fi
cargo test --locked --offline --manifest-path "$fixture/Cargo.toml"
cargo test --locked --offline --manifest-path "$fixture/Cargo.toml" --features wire-schema
cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" --features wire-schema \
    --edges normal,build --prefix none > "$fixture/host-schema.txt"
rg -q '^ic-wire-derive v' "$fixture/host-schema.txt"
for features in '' wire-schema; do
    # Bash 3.2 treats an empty array as unset under nounset. Keep the shared
    # target arguments in the array so both feature selections remain nonempty.
    options=(--target wasm32-unknown-unknown)
    [[ -z "$features" ]] || options+=(--features "$features")
    cargo check --locked --offline --manifest-path "$fixture/Cargo.toml" "${options[@]}"
    cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" \
        -e normal --prefix none --format '{p}' "${options[@]}" > "$fixture/wasm-${features:-runtime}.txt"
    if rg '^serde_json v' "$fixture/wasm-${features:-runtime}.txt"; then
        echo 'Host metadata dependency leaked into the Wasm runtime graph' >&2
        exit 1
    fi
    cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" \
        --edges normal,build --prefix none "${options[@]}" > "$fixture/wasm-${features:-runtime}-build.txt"
    if [[ -z "$features" ]] && rg '^ic-wire-derive v' "$fixture/wasm-runtime-build.txt"; then
        echo 'Wasm codec-only consumer selected the metadata derive' >&2
        exit 1
    fi
    # ByteBuf is a runtime dependency now; keep its alloc-only feature boundary.
    cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" \
        -e normal --prefix none --format '{p} features={f}' "${options[@]}" > "$fixture/wasm-${features:-runtime}-features.txt"
    if ! rg -q '^serde_bytes v[^ ]+ features=alloc$' "$fixture/wasm-${features:-runtime}-features.txt"; then
        echo 'ByteBuf requires the alloc-only serde_bytes runtime dependency' >&2
        exit 1
    fi
done
printf 'Native metadata and Wasm schema forwarding passed; evidence: %s\n' "$fixture"

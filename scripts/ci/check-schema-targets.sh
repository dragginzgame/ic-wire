#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target/schema-targets"
fixture="$(mktemp -d "$root/target/schema-targets/consumer.XXXXXX")"
printf 'Schema forwarding fixture: %s\n' "$fixture"
cp "$root/crates/ic-wire-qualification/fixtures/schema_targets/mod.rs" "$fixture/lib.rs"
# Start with the selected workspace versions; resolution may only update this
# disposable consumer lock. Every build/check after resolution is locked/offline.
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cat > "$fixture/Cargo.toml" <<EOF_MANIFEST
[workspace]
[package]
name = "wire-schema-targets"
version = "0.0.0"
edition = "2024"
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
cargo test --locked --offline --manifest-path "$fixture/Cargo.toml"
cargo test --locked --offline --manifest-path "$fixture/Cargo.toml" --features wire-schema
for features in '' wire-schema; do
    options=()
    [[ -z "$features" ]] || options=(--features "$features")
    cargo check --locked --offline --manifest-path "$fixture/Cargo.toml" --target wasm32-unknown-unknown "${options[@]}"
    cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" --target wasm32-unknown-unknown \
        -e normal --prefix none --format '{p}' "${options[@]}" > "$fixture/wasm-${features:-runtime}.txt"
    if rg '^serde_json v' "$fixture/wasm-${features:-runtime}.txt"; then
        echo 'Host metadata dependency leaked into the Wasm runtime graph' >&2
        exit 1
    fi
    # ByteBuf is a runtime dependency now; keep its alloc-only feature boundary.
    cargo tree --locked --offline --manifest-path "$fixture/Cargo.toml" --target wasm32-unknown-unknown \
        -e normal --prefix none --format '{p} features={f}' "${options[@]}" > "$fixture/wasm-${features:-runtime}-features.txt"
    if ! rg -q '^serde_bytes v[^ ]+ features=alloc$' "$fixture/wasm-${features:-runtime}-features.txt"; then
        echo 'ByteBuf requires the alloc-only serde_bytes runtime dependency' >&2
        exit 1
    fi
done
RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked --offline --manifest-path "$fixture/Cargo.toml" \
    --target wasm32-unknown-unknown --features wire-schema
printf 'Native metadata and Wasm schema forwarding passed; evidence: %s\n' "$fixture"

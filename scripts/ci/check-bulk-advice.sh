#!/usr/bin/env bash
# Qualify ordinary Cargo build output, not only the diagnostic data structure.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target/bulk-advice"
fixture="$(mktemp -d "$root/target/bulk-advice/consumer.XXXXXX")"
cp "$root/qualification/ic-wire-qualification/tests/fixtures/bulk_advice/build.rs" "$fixture/build.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
: > "$fixture/lib.rs"
cat > "$fixture/Cargo.toml" <<EOF_MANIFEST
[workspace]
[package]
name = "wire-bulk-advice"
version = "0.0.0"
edition = "2024"
rust-version = "$RUSTUP_TOOLCHAIN"
publish = false
[lib]
path = "lib.rs"
[build-dependencies]
ic-wire = { path = "$root/crates/ic-wire", features = ["typescript"] }
serde_json = "1.0"
EOF_MANIFEST
export CARGO_TARGET_DIR="$root/target/bulk-advice/build"
cargo metadata --offline --format-version 1 --manifest-path "$fixture/Cargo.toml" > "$fixture/metadata.json"
bash "$root/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture" --external-ic
for mode in default partial all stale; do
    WIRE_ADVICE_CASE="$mode" cargo check --locked --offline --manifest-path "$fixture/Cargo.toml" > "$fixture/$mode.log" 2>&1
done
byte='warning: wire-bulk-advice@0.0.0: /types/Payload/fields/data/list/u8: consider'
word='warning: wire-bulk-advice@0.0.0: /types/Payload/fields/words/list/u16: consider'
rg -Fq "$byte" "$fixture/default.log"
rg -Fq "$word" "$fixture/default.log"
rg -Fq "/types/Payload/fields/ids/list/hash16: consider Packed<[u8; 16]>" "$fixture/default.log"
rg -Fq '/types/Payload/fields/flags/list/bool: consider' "$fixture/default.log"
for entry in 'signed/list/i32: consider Packed<i32>' 'hashes/list/hash32: consider Packed<[u8; 32]>' 'samples32/list/f32: consider the explicit finite_f32 adapter' 'samples64/list/f64: consider the explicit finite_f64 adapter'; do
    rg -Fq "/types/Payload/fields/$entry" "$fixture/default.log"
done
rg -Fq "$word" "$fixture/partial.log"
if rg -F "$byte" "$fixture/partial.log" || rg 'warning:' "$fixture/all.log"; then
    echo 'Acknowledged field still emitted a warning' >&2
    exit 1
fi
rg -Fq 'warning: wire-bulk-advice@0.0.0: /removed/field: stale acknowledgement' "$fixture/stale.log"
if rg ': consider ' "$fixture/stale.log"; then
    echo 'Stale acknowledgement unexpectedly restored an acknowledged warning' >&2
    exit 1
fi
printf 'Cargo bulk advice visibility/suppression passed; evidence: %s\n' "$fixture"

#!/usr/bin/env bash
# Verify shipped sources without publishing the coordinated derive dependency.
set -euo pipefail
mkdir -p target/package-check
output=$(mktemp -d "$PWD/target/package-check/run-XXXXXX")
cargo metadata --no-deps --format-version 1 --offline > "$output/metadata.json"
version=$(node -e 'const fs=require("node:fs"); const m=JSON.parse(fs.readFileSync(process.argv[1])); console.log(m.packages.find(p=>p.name==="ic-wire").version)' "$output/metadata.json")
cargo package --workspace --all-features --allow-dirty --no-verify --offline
for package in ic-wire ic-wire-derive; do
    tar -xzf "target/package/$package-$version.crate" -C "$output"
    cmp LICENSE.md "$output/$package-$version/LICENSE.md"
    test -s "$output/$package-$version/README.md"
done
node qualification/prepare-package-consumer.mjs "$output" "$version"
config="$output/patch.toml"
export CARGO_TARGET_DIR="$output/build"
# Cargo's workspace temporary-registry verifier currently fails with "no hash
# listed" for the unpublished derive. Check the extracted archives explicitly.
cargo test --manifest-path "$output/ic-wire-$version/Cargo.toml" --all-features --offline --config "$config"
cargo test --manifest-path "$output/ic-wire-consumer/Cargo.toml" --test contract --features schema --offline --config "$config"
cargo test --manifest-path "$output/ic-wire-consumer/Cargo.toml" --test package_platform --features schema,wire/platform-types --offline --config "$config"
cargo build --manifest-path "$output/ic-wire-consumer/Cargo.toml" --bin wire-consumer --target wasm32-unknown-unknown --offline --config "$config"
node qualification/ic-wire-consumer/check-wasm.mjs "$output/build/wasm32-unknown-unknown/debug/wire-consumer.wasm"
cargo check --manifest-path "$output/ic-wire-$version/Cargo.toml" --all-features --lib --target wasm32-unknown-unknown --offline --config "$config"
cargo run --manifest-path "$output/ic-wire-consumer/Cargo.toml" --features schema --bin wire-consumer-generate --offline --config "$config" -- "$output/client"
printf 'Verified package consumer and generated clients: %s\n' "$output"

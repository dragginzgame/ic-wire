#!/usr/bin/env bash
# Published-supplier composition only; never edits the supplying Blob checkout.
set -euo pipefail
[[ $# == 2 && "$1" == /* && "$2" == /* ]] || { echo 'usage: reproduce.sh /fresh/output /blob/repository' >&2; exit 2; }
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
output="$1"
blob="$2"
mkdir "$output"
mkdir -p "$output/consumer/src" "$output/consumer/browser" "$output/blob"
cp "$root/docs/evidence/published-pair-0110/consumer-Cargo.toml.snapshot" "$output/consumer/Cargo.toml"
cp "$root/crates/ic-wire-qualification/fixtures/platform_types/mod.rs" "$output/consumer/src/lib.rs"
export CARGO_TARGET_DIR="$output/target" RUSTUP_AUTO_INSTALL=0
export WIRE_PLATFORM_TYPES_OUTPUT="$output/consumer/browser"
cargo +1.91.0 test --manifest-path "$output/consumer/Cargo.toml" --offline --features endpoint --lib > "$output/rust-tests.log" 2>&1
cargo +1.91.0 check --manifest-path "$output/consumer/Cargo.toml" --locked --offline --features endpoint --target wasm32-unknown-unknown > "$output/wasm-check.log" 2>&1
cargo metadata --manifest-path "$output/consumer/Cargo.toml" --locked --offline --features endpoint --format-version 1 > "$output/metadata.json"
for selection in schema endpoint; do
    options=()
    [[ "$selection" != endpoint ]] || options+=(--features endpoint)
    for target in "$(rustc +1.91.0 -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        graph="$output/$selection-$target.txt"
        cargo tree --manifest-path "$output/consumer/Cargo.toml" --locked --offline "${options[@]}" --target "$target" --edges normal,build --prefix none --format '{p} features={f}' > "$graph"
        if grep -Eq '^(candid|ic-cdk|ic-cdk-executor|pocket-ic) v' "$graph"; then exit 1; fi
        if [[ "$selection" == schema ]]; then
            if grep -Eq '^ic-platform-(macros|runtime-guard) v|^ic-platform v.*features=.*(cdk|executor|candid)' "$graph"; then exit 1; fi
        else
            grep -q '^ic-platform-runtime-guard v0.12.0' "$graph"
        fi
        if [[ "$target" == wasm32-unknown-unknown ]] && grep -q '^serde_json v' "$graph"; then exit 1; fi
    done
done
printf '{"type":"module"}\n' > "$output/consumer/browser/package.json"
node "$root/examples/browser/node_modules/typescript/bin/tsc" --ignoreConfig --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022 --module NodeNext --moduleResolution NodeNext --outDir "$output/consumer/browser" "$output/consumer/browser/principal.ts" "$output/consumer/browser/account.ts" "$output/consumer/browser/wire-runtime.ts"
cp "$root/crates/ic-wire-qualification/fixtures/platform_types/check.mjs" "$output/consumer/browser/check.mjs"
node "$output/consumer/browser/check.mjs" > "$output/browser-check.log"
git -C "$blob" archive 7fe95703365d51ceda1c0b37d35ca42882ec812d | tar -x -C "$output/blob"
node - "$output" <<'JS'
const fs = require('node:fs');
const output = process.argv[2];
const metadata = JSON.parse(fs.readFileSync(`${output}/metadata.json`));
for (const name of ['ic-wire', 'ic-wire-derive', 'ic-platform', 'ic-platform-macros', 'ic-platform-runtime-guard']) {
    const packages = metadata.packages.filter(p => p.name === name);
    if (packages.length !== 1 || !packages[0].source?.startsWith('registry+')) throw Error(`non-registry or duplicate ${name}`);
}
const manifest = `${output}/blob/Cargo.toml`;
let source = fs.readFileSync(manifest, 'utf8');
for (const [old, selected] of [
    ['ic-wire = "0.10.0"', 'ic-wire = "=0.11.0"'],
    ['ic-platform = { version = "0.11.1",', 'ic-platform = { version = "=0.12.0",'],
    ['ic-metrics = "0.21.0"', 'ic-metrics = "=0.22.0"'],
]) {
    if (!source.includes(old)) throw Error(`unexpected released manifest: ${old}`);
    source = source.replace(old, selected);
}
fs.writeFileSync(manifest, source);
JS
cargo +1.91.0 check --manifest-path "$output/blob/Cargo.toml" --offline -p ic-blob-storage-contracts --features wire-schema --example wire_schema > "$output/blob-check.log" 2>&1
printf 'Published Wire/Platform/Metrics pairing passed; evidence: %s\n' "$output"

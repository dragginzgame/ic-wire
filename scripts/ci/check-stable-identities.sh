#!/usr/bin/env bash
# Future crypto-facade simulation; the execution owner is held fixed.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
out="$(mktemp -d "$root/target/stable-identities.XXXXXX")"
printf 'Stable identity evidence: %s\n' "$out"
cargo metadata --locked --offline --features ic-wire/platform-types,ic-wire-benchmarks/identity-fixture --format-version 1 > "$out/workspace.json"
node scripts/ci/stable-identity-fixtures.mjs prepare "$root" "$out"
for owner in next-platform next-endpoint next-endpoint-macros; do
    bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$out/$owner" --external-ic
done
export CARGO_TARGET_DIR="$out/target"
for lane in current next; do
    manifest="$out/$lane/Cargo.toml"
    cargo metadata --manifest-path "$manifest" --offline --format-version 1 > "$out/$lane/metadata.json"
    bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$out/$lane" --external-ic
    mkdir "$out/$lane/browser"
    WIRE_PLATFORM_TYPES_OUTPUT="$out/$lane/browser" cargo test --manifest-path "$manifest" --locked --offline --lib
    node scripts/ci/check-ic-sdk-boundary.mjs "$out/$lane/browser"
    cargo check --manifest-path "$manifest" --locked --offline --target wasm32-unknown-unknown
    cargo tree --manifest-path "$manifest" --locked --offline --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/$lane/graph.txt"
    [[ "$(grep -c '^ic-platform-runtime-guard v' "$out/$lane/graph.txt")" == 1 ]]
    if grep -Eq '^(candid|pocket-ic|ic-platform-pocket-ic|ic-testkit|ic-cdk-executor|serde_json) v' "$out/$lane/graph.txt"; then exit 1; fi
done
node scripts/ci/stable-identity-fixtures.mjs verify "$root" "$out"

#!/usr/bin/env bash
# Published/current mixtures; patches are confined to disposable qualification.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target/compatible-consumers"
out="$(mktemp -d "$root/target/compatible-consumers/run.XXXXXX")"
# Previous supported Core and currently published Endpoint. These are test inputs,
# not production pins; keep them old when advancing the candidate patch.
for lane in current previous-core published-endpoint; do
    fixture="$out/$lane"
    mkdir -p "$fixture/src"
    cp "$root/crates/ic-wire-qualification/tests/fixtures/compatibility/mod.rs" "$fixture/src/lib.rs"
    cp "$root/crates/ic-wire-qualification/benches/contract_tests.rs" "$fixture/src/contract_tests.rs"
    cp "$root/Cargo.lock" "$fixture/Cargo.lock"
    node - "$root" "$fixture" "$lane" <<'JS'
const fs = require('node:fs');
const [root, fixture, lane] = process.argv.slice(2);
const local = name => `path = ${JSON.stringify(`${root}/crates/${name}`)}`;
const core = lane === 'previous-core' ? 'version = "=0.17.2"' : local('ic-wire');
const endpoint = lane === 'current' ? local('ic-wire-endpoint') : 'version = "=0.7.1"';
const patches = [`ic-wire-derive = { ${local('ic-wire-derive')} }`];
if (lane !== 'previous-core') patches.push(`ic-wire = { ${local('ic-wire')} }`);
if (lane !== 'published-endpoint') patches.push(`ic-wire-endpoint-macros = { ${local('ic-wire-endpoint-macros')} }`);
fs.writeFileSync(`${fixture}/Cargo.toml`, `[workspace]
[package]
name = "wire-compatible-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "${process.env.RUSTUP_TOOLCHAIN}"
publish = false
[dependencies]
ic-wire = { ${core}, features = ["platform-types"] }
ic-wire-endpoint = { ${endpoint}, features = ["typescript"] }
ic-platform-types = { version = "0.18.1", default-features = false }
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
[patch.crates-io]
${patches.join('\n')}
`);
JS
    export CARGO_TARGET_DIR="$out/target-$lane"
    # Prepare the exact historical packages when absent from the Cargo cache;
    # only the disposable lock may change. Remaining commands stay locked/offline.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --format-version 1 > "$fixture/metadata.json"
    bash "$root/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture" --external-ic
    node - "$fixture/metadata.json" "$lane" <<'JS'
const fs = require('node:fs'), assert = require('node:assert/strict');
const [file, lane] = process.argv.slice(2), m = JSON.parse(fs.readFileSync(file));
for (const name of ['ic-wire', 'ic-wire-derive', 'ic-wire-endpoint', 'ic-wire-endpoint-macros', 'ic-platform-types', 'ic-platform-runtime', 'ic-platform-runtime-guard']) {
 const matches = m.packages.filter(p => p.name === name);
 assert.equal(matches.length, 1, `duplicate/missing ${name}`);
 const p = matches[0];
 if (name === 'ic-wire' && lane === 'previous-core') { assert.equal(p.version,'0.17.2'); assert.ok(p.source?.startsWith('registry+')); }
 if (name === 'ic-wire-derive' || (name === 'ic-wire' && lane !== 'previous-core')) assert.equal(p.source,null);
 if (name === 'ic-wire-endpoint') assert.equal(p.source === null,lane === 'current');
 if (name === 'ic-wire-endpoint-macros') assert.equal(p.source === null,lane !== 'published-endpoint');
}
assert.ok(!m.packages.some(p => ['candid','ic-testkit','tokio'].includes(p.name)));
JS
    cargo test --manifest-path "$fixture/Cargo.toml" --locked --offline --lib
    cargo check --manifest-path "$fixture/Cargo.toml" --locked --offline --target wasm32-unknown-unknown
    printf 'Compatible consumer passed: %s\n' "$lane"
done
printf 'Mixed-version evidence: %s\n' "$out"

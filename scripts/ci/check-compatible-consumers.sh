#!/usr/bin/env bash
# Published/current mixtures; patches are confined to disposable qualification.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target/compatible-consumers"
out="$(mktemp -d "$root/target/compatible-consumers/run.XXXXXX")"
# Qualify current packages and independently published Endpoint. No previous-Core
# support lane: public hard cuts migrate consumers to the current Core contract.
for lane in current published-endpoint; do
    fixture="$out/$lane"
    mkdir -p "$fixture/src"
    cp "$root/crates/ic-wire-qualification/tests/fixtures/compatibility/mod.rs" "$fixture/src/lib.rs"
    cp "$root/crates/ic-wire-qualification/benches/contract_tests.rs" "$fixture/src/contract_tests.rs"
    cp "$root/Cargo.lock" "$fixture/Cargo.lock"
    node - "$root" "$fixture" "$lane" <<'JS'
const fs = require('node:fs');
const [root, fixture, lane] = process.argv.slice(2);
const local = name => `path = ${JSON.stringify(`${root}/crates/${name}`)}`;
const core = local('ic-wire');
// Follow the independent Endpoint minor series without requiring an unpublished
// patch during release preparation. Cargo selects its published patch.
const manifest = fs.readFileSync(root + '/crates/ic-wire-endpoint/Cargo.toml', 'utf8');
const version = manifest.match(/^version = "(\d+\.\d+)\.\d+"$/m)?.[1];
if (!version) throw new Error('missing literal Endpoint version');
const endpoint = lane === 'current' ? local('ic-wire-endpoint') : `version = "${version}"`;
const patches = [`ic-wire-derive = { ${local('ic-wire-derive')} }`];
patches.push(`ic-wire = { ${local('ic-wire')} }`);
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
const currentCore = m.packages.find(p => p.name === 'ic-wire' && p.source === null);
assert.ok(currentCore, 'missing current Core');
const endpoint = m.packages.find(p => p.name === 'ic-wire-endpoint');
const dependency = m.resolve.nodes.find(n => n.id === endpoint.id).deps.find(d => d.name === 'ic_wire');
const selectedCore = m.packages.find(p => p.id === dependency.pkg);
const line = v => { const [major, minor] = v.split('.'); return major === '0' ? major + '.' + minor : major; };
assert.ok(!m.packages.some(p => ['candid','ic-testkit','tokio'].includes(p.name)));
for (const name of ['ic-wire-endpoint', 'ic-wire-endpoint-macros', 'ic-platform-types', 'ic-platform-runtime', 'ic-platform-runtime-guard']) {
 assert.equal(m.packages.filter(p => p.name === name).length, 1, `duplicate/missing ${name}`);
}
if (lane === 'published-endpoint' && line(selectedCore.version) !== line(currentCore.version)) {
 // A pre-1.0 Core minor cut does not promise compatibility with old Endpoint.
 // Keep the current pair qualified, and never mistake a same-line identity split
 // for that intentional boundary. Bytes are unchanged; align consumer versions.
 assert.notEqual(endpoint.source, null);
 assert.notEqual(selectedCore.source, null);
 for (const name of ['ic-wire', 'ic-wire-derive']) {
  const versions = m.packages.filter(p => p.name === name);
  assert.equal(versions.length, 2, `unexpected identity split for ${name}`);
  assert.equal(versions.filter(p => line(p.version) === line(currentCore.version)).length, 1);
  assert.equal(versions.filter(p => line(p.version) === line(selectedCore.version)).length, 1);
 }
 const requirement = endpoint.dependencies.find(d => d.name === 'ic-wire').req;
 console.log(`Published Endpoint ${endpoint.version} requires Core ${requirement}; working Core ${currentCore.version} is a different compatibility line. Current pair is qualified separately.`);
 fs.writeFileSync(file + '.unsupported-core-line', selectedCore.version + '\n');
 process.exit(0);
}
for (const name of ['ic-wire', 'ic-wire-derive', 'ic-wire-endpoint', 'ic-wire-endpoint-macros', 'ic-platform-types', 'ic-platform-runtime', 'ic-platform-runtime-guard']) {
 const matches = m.packages.filter(p => p.name === name);
 assert.equal(matches.length, 1, `duplicate/missing ${name}`);
 const p = matches[0];
 if (name === 'ic-wire-derive' || name === 'ic-wire') assert.equal(p.source,null);
 if (name === 'ic-wire-endpoint') assert.equal(p.source === null,lane === 'current');
 if (name === 'ic-wire-endpoint-macros') assert.equal(p.source === null,lane !== 'published-endpoint');
}
JS
    if [[ -f "$fixture/metadata.json.unsupported-core-line" ]]; then
        continue
    fi
    cargo test --manifest-path "$fixture/Cargo.toml" --locked --offline --lib
    cargo check --manifest-path "$fixture/Cargo.toml" --locked --offline --target wasm32-unknown-unknown
    printf 'Compatible consumer passed: %s\n' "$lane"
done
printf 'Mixed-version evidence: %s\n' "$out"

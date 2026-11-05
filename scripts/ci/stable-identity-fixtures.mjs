// Prepare and compare the standalone identity consumers; never edit suppliers.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
const [mode, root, out] = process.argv.slice(2);
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const write = (file, text) => fs.writeFileSync(file, text);
if (mode === 'prepare') {
  const m = read(path.join(out, 'workspace.json'));
  const local = name => m.packages.find(p => p.manifest_path === path.join(root, 'crates', name, 'Cargo.toml'));
  const dependency = (owner, name) => m.packages.find(p => p.id === m.resolve.nodes.find(n => n.id === owner.id).deps.find(d => d.name === name).pkg);
  const selection = p => p.source ? `version = "=${p.version}"` : `path = ${JSON.stringify(path.dirname(p.manifest_path))}`;
  const next = version => { const [major, minor] = version.split('.').map(Number); return `${major}.${minor + 1}.0`; };
  const wire = local('ic-wire'), endpoint = local('ic-wire-endpoint');
  const types = dependency(wire, 'ic_platform_types');
  const platform = dependency(local('ic-wire-benchmarks'), 'ic_platform');
  const runtime = dependency(endpoint, 'ic_platform_runtime');
  const futureEndpoint = next(endpoint.version);
  const nextPlatform = path.join(out, 'next-platform');
  fs.cpSync(path.dirname(platform.manifest_path), nextPlatform, { recursive: true });
  const manifest = path.join(nextPlatform, 'Cargo.toml');
  const original = fs.readFileSync(manifest, 'utf8'), version = `version = "${platform.version}"`;
  assert.ok(original.includes(version));
  write(manifest, original.replace(version, `version = "${next(platform.version)}"`));
  const macros = path.join(out, 'next-endpoint-macros');
  fs.cpSync(path.join(root, 'crates/ic-wire-endpoint-macros/src'), path.join(macros, 'src'), { recursive: true });
  write(path.join(macros, 'Cargo.toml'), `[workspace]
[package]
name = "ic-wire-endpoint-macros"
version = "${futureEndpoint}"
edition = "2024"
rust-version = "${wire.rust_version}"
[lib]
proc-macro = true
[dependencies]
proc-macro2 = "1.0.107"
quote = "1.0.47"
syn = { version = "3.0.9", features = ["full"] }
`);
  const nextEndpoint = path.join(out, 'next-endpoint');
  fs.cpSync(path.join(root, 'crates/ic-wire-endpoint/src'), path.join(nextEndpoint, 'src'), { recursive: true });
  write(path.join(nextEndpoint, 'Cargo.toml'), `[workspace]
[package]
name = "ic-wire-endpoint"
version = "${futureEndpoint}"
edition = "2024"
rust-version = "${wire.rust_version}"
[features]
typescript = ["ic-wire/typescript", "dep:serde_json"]
[target.'cfg(not(target_family = "wasm"))'.dependencies]
serde_json = { version = "1", optional = true }
[dependencies]
ic-platform-runtime = { ${selection(runtime)}, default-features = false, features = ["cdk"] }
ic-wire = { path = ${JSON.stringify(path.join(root, 'crates/ic-wire'))} }
ic-wire-endpoint-macros = { path = ${JSON.stringify(macros)}, version = "=${futureEndpoint}" }
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
`);
  for (const lane of ['current', 'next']) {
    const directory = path.join(out, lane); fs.mkdirSync(path.join(directory, 'src'), { recursive: true });
    fs.copyFileSync(path.join(root, 'Cargo.lock'), path.join(directory, 'Cargo.lock'));
    const fixtures = path.join(root, 'crates/ic-wire-qualification/tests/fixtures/platform_types');
    fs.copyFileSync(path.join(fixtures, 'mod.rs'), path.join(directory, 'src/lib.rs'));
    fs.cpSync(path.join(fixtures, 'tests'), path.join(directory, 'src/tests'), { recursive: true });
    const facade = lane === 'current' ? `version = "=${platform.version}"` : `path = ${JSON.stringify(nextPlatform)}`;
    const endpointPath = lane === 'current' ? path.join(root, 'crates/ic-wire-endpoint') : nextEndpoint;
    write(path.join(directory, 'Cargo.toml'), `[workspace]
[package]
name = "wire-stable-identity-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "${wire.rust_version}"
[features]
default = ["endpoint"]
endpoint = []
[dependencies]
ic-wire = { path = ${JSON.stringify(path.join(root, 'crates/ic-wire'))}, features = ["platform-types"] }
ic-wire-endpoint = { path = ${JSON.stringify(endpointPath)}, features = ["typescript"] }
ic-platform = { ${facade}, default-features = false, features = ["blake3"] }
ic-platform-types = { ${selection(types)}, default-features = false }
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
[dev-dependencies]
serde_bytes = "0.11"
serde_json = "1"
`);
  }
} else if (mode === 'verify') {
  const metadata = ['current', 'next'].map(lane => read(path.join(out, lane, 'metadata.json')));
  const prepared = new Set(read(path.join(out, 'workspace.json')).packages.filter(p => p.source).map(p => JSON.stringify([p.name, p.version, p.source])));
  for (const m of metadata) for (const p of m.packages.filter(p => p.source)) assert.ok(prepared.has(JSON.stringify([p.name, p.version, p.source])), 'identity fixture requires unprepared registry dependencies');
  const identities = {}, selected = name => metadata.map(m => m.packages.filter(p => p.name === name).map(p => p.id));
  for (const name of ['ic-wire', 'ic-wire-derive', 'ic-platform-types', 'ic-platform-runtime', 'ic-platform-macros', 'ic-platform-runtime-guard']) {
    const ids = selected(name); assert.equal(ids[0].length, 1); assert.deepEqual(ids[0], ids[1]); identities[name] = ids[0][0];
  }
  for (const name of ['ic-platform', 'ic-wire-endpoint', 'ic-wire-endpoint-macros']) {
    const ids = selected(name); assert.equal(ids[0].length, 1); assert.equal(ids[1].length, 1); assert.notDeepEqual(ids[0], ids[1]); identities[name] = { current: ids[0], 'simulated-next': ids[1] };
  }
  write(path.join(out, 'identities.json'), JSON.stringify(identities, null, 2) + '\n');
  console.log('Codec/schema/derive and Principal/Account supplier IDs unchanged across simulated endpoint/crypto-facade generation.');
} else {
  throw new Error('select prepare ROOT OUTPUT or verify ROOT OUTPUT');
}

// Wire-owned qualification graphs; no builds, simulator startup or supplier waits.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export function graph(text) {
  const result = new Map();
  for (const line of text.trim().split('\n').filter(Boolean)) {
    const [identity, features] = line.replace(/ \(\*\)$/, '').split('|');
    assert.ok(identity && features !== undefined, `invalid Cargo graph row: ${line}`);
    const name = identity.split(' ')[0];
    const entries = result.get(name) ?? new Map();
    entries.set(identity, new Set([...(entries.get(identity) ?? []), ...features.split(',').filter(Boolean)]));
    result.set(name, entries);
  }
  return result;
}
export function admit(packages, { native = false, wire, types, runtime, typed } = {}) {
  const one = (name, selected = packages) => {
    const entries = selected.get(name);
    assert.equal(entries?.size, 1, `expected one ${name} identity`);
    return [...entries][0];
  };
  if (native) {
    assert.ok(typed instanceof Map && !typed.has('ic-testkit'), 'missing supplier-pruned typed graph');
    assert.equal(one('ic-wire', typed)[0], wire, 'typed boundary must use local Wire');
    assert.ok(!one('ic-testkit')[1].has('wire'), 'Testkit Wire adapter must remain disabled');
    assert.equal(one('ic-wire')[0], wire, 'qualification must use the local Wire codec');
  } else {
    for (const name of ['ic-testkit', 'ic-build', 'dhat', 'stats_alloc']) {
      assert.ok(!packages.has(name), `${name} must stay outside this graph`);
    }
  }
  if (types && packages.has('ic-platform-types')) {
    assert.equal(one('ic-platform-types', native ? typed : packages)[0], types, 'canonical Types identity differs');
  }
  if (runtime && packages.has('ic-platform-runtime')) {
    const [identity, features] = one('ic-platform-runtime');
    assert.equal(identity, runtime, 'qualification Runtime identity split');
    if (features.has('cdk') || features.has('runtime')) one('ic-platform-runtime-guard');
  }
}

function main() {
  const root = fs.realpathSync(path.join(path.dirname(fileURLToPath(import.meta.url)), '../..'));
  const run = (cmd, args) => execFileSync(cmd, args, { cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
  const integration = process.argv.includes('--integration');
  const manifest = integration ? ['--manifest-path', 'qualification/Cargo.toml'] : [];
  const cargo = args => run('cargo', [...args, ...manifest, '--locked', '--offline']);
  const workspace = JSON.parse(cargo(['metadata', '--format-version', '1', ...(!integration ? ['--all-features'] : [])]));
  const member = name => workspace.packages.find(p => p.name === name);
  const members = workspace.packages.filter(p => workspace.workspace_members.includes(p.id));
  if (integration) {
    const product = JSON.parse(run('cargo', ['metadata', '--locked', '--offline', '--all-features', '--format-version', '1']));
    const core = product.packages.find(p => p.name === 'ic-wire');
    const tooling = product.packages.find(p => p.name === 'toko-tooling');
    assert.equal(member('toko-tooling').id, tooling.id, 'qualification must select the admitted tooling');
    for (const pkg of members) {
      assert.deepEqual(pkg.publish, [], 'qualification must remain unpublished');
      assert.equal(pkg.rust_version, core.rust_version, 'qualification compiler floor differs');
      assert.equal(pkg.dependencies.find(d => d.name === 'toko-tooling')?.req, `=${tooling.version}`);
    }
  }
  const harnesses = ['ic-wire-benchmarks', 'ic-wire-endpoint-fixture'];
  if (!integration) {
    for (const name of ['ic-build', 'ic-testkit', 'ic-platform', 'tokio']) {
      assert.ok(!member(name), `${name} must not constrain the product lockfile`);
    }
  }
  for (const pkg of members) {
    const deps = pkg.dependencies.filter(d => d.name === 'ic-testkit');
    assert.equal(deps.length, harnesses.includes(pkg.name) ? 1 : 0, `${pkg.name}: unexpected Testkit declaration`);
    for (const dep of deps) {
      assert.deepEqual(pkg.publish, [], `${pkg.name}: qualification package must remain private`);
      assert.equal(dep.kind, 'dev');
      assert.equal(dep.optional, false);
      assert.ok(!pkg.features.ic.includes('dep:ic-testkit'));
      assert.equal(dep.target, 'cfg(not(target_family = "wasm"))');
      assert.equal(dep.uses_default_features, false);
      assert.ok(!dep.features.includes('wire'));
    }
  }
  fs.mkdirSync(path.join(root, 'target'), { recursive: true });
  const out = fs.mkdtempSync(path.join(root, integration ? 'target/integration-boundary.' : 'target/product-boundary.'));
  const host = run('rustc', ['-vV']).match(/^host: (.+)$/m)[1];
  const graphs = [];
  const select = (name, features, target, dev = false, pruneTestkit = false) => {
    const args = ['tree', '-p', name, '--target', target, '--edges', dev ? 'normal,build,dev' : 'normal,build', '--prefix', 'none', '--format', '{p}|{f}'];
    if (features) args.push('--features', features);
    if (pruneTestkit) args.push('--prune', 'ic-testkit');
    const text = integration && !members.some(p => p.name === name)
      ? run('cargo', [...args, '--locked', '--offline']) : cargo(args);
    const parsed = graph(text);
    const file = `${graphs.length}.txt`;
    fs.writeFileSync(path.join(out, file), text);
    graphs.push({ package: name, features, target, dev, pruneTestkit, file });
    return parsed;
  };
  const schema = select('ic-wire', 'platform-types', host);
  const endpoint = select('ic-wire-endpoint', '', host);
  const identity = (g, name) => {
    assert.equal(g.get(name)?.size, 1, `ambiguous selected ${name}`);
    return [...g.get(name).keys()][0];
  };
  const expected = {
    wire: `ic-wire v${member('ic-wire').version} (${path.dirname(member('ic-wire').manifest_path)})`,
    types: identity(schema, 'ic-platform-types'), runtime: identity(endpoint, 'ic-platform-runtime'),
  };
  admit(schema); admit(endpoint, expected);
  for (const pkg of members) {
    for (const dep of pkg.dependencies.filter(d => d.name === 'ic-platform-types')) {
      assert.equal(dep.req, member('ic-wire').dependencies.find(d => d.name === 'ic-platform-types').req,
        `${pkg.name}: local canonical Types requirement differs from Core`);
    }
  }
  for (const target of [host, 'wasm32-unknown-unknown']) {
    for (const name of ['ic-wire', 'ic-wire-derive', 'ic-wire-endpoint', 'ic-wire-endpoint-macros']) {
      admit(select(name, '', target), expected);
    }
  }
  for (const name of integration ? harnesses : []) {
    const selections = name === 'ic-wire-benchmarks' ? ['', 'wire,candid,ic', 'ic,endpoint-input'] : ['', 'ic,schema'];
    for (const features of selections) {
      // Older identities may remain inside Testkit's native-only supplier tree.
      // Pruning that owner retains every direct/local typed path; the full graph
      // still enforces the Wire identity and Runtime singleton.
      const typed = select(name, features, host, true, true);
      admit(select(name, features, host, true), { ...expected, native: true, typed });
      admit(select(name, features, 'wasm32-unknown-unknown', true), expected);
    }
  }
  if (integration) {
    admit(select('ic-wire-qualification', 'allocation-fixture,schema-fixture', 'wasm32-unknown-unknown', true), expected);
  }
  const sources = {};
  const hash = file => { sources[path.relative(root, file)] = crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'); };
  const walk = directory => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(file); else if (entry.isFile()) hash(file);
    }
  };
  for (const name of ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'scripts/ci/check-testkit-boundary.mjs', ...(integration ? ['qualification/Cargo.toml', 'qualification/Cargo.lock'] : [])]) hash(path.join(root, name));
  for (const pkg of workspace.packages.filter(p => !p.source)) {
    hash(pkg.manifest_path);
    for (const directory of ['src', 'benches', 'tests', 'tools']) {
      const file = path.join(path.dirname(pkg.manifest_path), directory);
      if (fs.existsSync(file)) walk(file);
    }
  }
  fs.writeFileSync(path.join(out, 'identity.json'), JSON.stringify({ host, expected, sources, graphs }, null, 2) + '\n');
  console.log(`Testkit boundary passed: ${graphs.length} locked graphs; local Wire/Types/Runtime identities recorded in ${out}`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();

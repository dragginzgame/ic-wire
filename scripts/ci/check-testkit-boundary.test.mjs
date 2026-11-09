import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { graph, admit } from './check-testkit-boundary.mjs';
const expected = { native: true, wire: 'ic-wire v1.0.0 (/local/wire)', types: 'ic-platform-types v2.0.0', runtime: 'ic-platform-runtime v3.0.0' };
const valid = `${expected.wire}|typescript\n${expected.types}|principal-external\nic-testkit v4.0.0|\n${expected.runtime}|cdk\nic-platform-runtime-guard v3.0.0|`;
test('raw native transport and runtime-only graphs remain admitted', () => {
  admit(graph(valid), expected);
  admit(graph(`${expected.wire}|`));
  admit(graph(valid + `\n${expected.wire}|typescript (*)`), expected);
});
for (const [name, text] of [
  ['adapter enabled', valid.replace('ic-testkit v4.0.0|', 'ic-testkit v4.0.0|wire')],
  ['registry Wire replaces local', valid.replace(' (/local/wire)', '')],
  ['second Wire', valid + '\nic-wire v0.9.0|'],
  ['second Types', valid + '\nic-platform-types v1.0.0|'],
  ['different Runtime', valid.replaceAll('ic-platform-runtime v3.0.0', 'ic-platform-runtime v2.0.0')],
  ['missing guard', valid.replace('\nic-platform-runtime-guard v3.0.0|', '')],
]) test(name, () => assert.throws(() => admit(graph(text), expected)));
test('Testkit is rejected in production or Wasm graphs', () => assert.throws(() => admit(graph(valid))));

test('an incompatible Testkit selection fails only the qualification workspace', () => {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  fs.mkdirSync(path.join(root, 'target'), { recursive: true });
  const fixture = fs.mkdtempSync(path.join(root, 'target/workspace-isolation.'));
  try {
    for (const name of ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'qualification/Cargo.toml', 'qualification/Cargo.lock']) {
      fs.mkdirSync(path.dirname(path.join(fixture, name)), { recursive: true });
      fs.copyFileSync(path.join(root, name), path.join(fixture, name));
    }
    for (const name of ['crates', 'qualification']) fs.cpSync(path.join(root, name), path.join(fixture, name), { recursive: true });
    const manifest = path.join(fixture, 'qualification/Cargo.toml');
    const original = fs.readFileSync(manifest, 'utf8');
    const broken = original.replace(/ic-testkit = \{ version = "[^"]+"/, 'ic-testkit = { version = "=0.0.0"');
    assert.notEqual(broken, original);
    fs.writeFileSync(manifest, broken);
    const observe = manifest => spawnSync('cargo', ['metadata', '--manifest-path', manifest, '--locked', '--offline', '--format-version', '1'],
      { cwd: fixture, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
    const product = observe('Cargo.toml');
    assert.ifError(product.error);
    assert.equal(product.status, 0, product.stderr);
    assert.ok(!JSON.parse(product.stdout).packages.some(p => p.name === 'ic-testkit'));
    const integration = observe('qualification/Cargo.toml');
    assert.ifError(integration.error);
    assert.notEqual(integration.status, 0);
    assert.match(integration.stderr, /ic-testkit/);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

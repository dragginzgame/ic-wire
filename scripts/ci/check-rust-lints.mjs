// Verify the actual manifest owners, then exercise Cargo inheritance negatively.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
function run(command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
const manifest = p => JSON.parse(run(path.join(root, '.tools/host/bin/yq'), ['-p', 'toml', '-o', 'json', '.', p]));
const product = manifest('Cargo.toml'), qualification = manifest('qualification/Cargo.toml');
const floor = product.workspace.package['rust-version'];
assert.equal(qualification.workspace.package['rust-version'], floor);
assert.equal(manifest('rust-toolchain.toml').toolchain.channel, floor);
assert.deepEqual(qualification.workspace.lints, product.workspace.lints);
for (const [base, owner] of [['.', product], ['qualification', qualification]]) {
  for (const member of owner.workspace.members) {
    const m = manifest(path.join(base, member, 'Cargo.toml'));
    assert.equal(m.package['rust-version'].workspace, true);
    if (m.package.name === 'ic-wire-benchmarks') {
      const expected = structuredClone(owner.workspace.lints);
      expected.rust.unsafe_code = 'deny';
      assert.deepEqual(m.lints, expected);
    } else assert.deepEqual(m.lints, { workspace: true });
  }
}
const lints = product.workspace.lints;
for (const name of ['dead_code', 'unused_must_use', 'unfulfilled_lint_expectations', 'unsafe_op_in_unsafe_fn']) assert.equal(lints.rust[name], 'deny');
assert.equal(lints.rust.unsafe_code, 'forbid');
assert.equal(lints.rust.missing_docs, 'warn');
assert.equal(lints.rustdoc.broken_intra_doc_links, 'deny');
assert.equal(lints.rustdoc.invalid_markdown_table, 'deny');
for (const name of ['unused_dependencies', 'unused_workspace_dependencies', 'unused_workspace_package_fields', 'missing_lints_inheritance', 'unknown_lints']) assert.equal(lints.cargo[name], 'deny');
assert.deepEqual(lints.clippy.all, { level: 'warn', priority: -2 });
assert.deepEqual(lints.clippy.correctness, { level: 'deny', priority: -1 });
for (const name of ['redundant_clone', 'needless_pass_by_value', 'wildcard_imports', 'missing_panics_doc', 'must_use_candidate']) assert.equal(lints.clippy[name], 'warn');
assert.equal(lints.clippy.too_many_lines, 'allow');
for (const name of ['nursery', 'restriction', 'future_not_send']) assert.equal(lints.clippy[name], undefined);

fs.mkdirSync('target', { recursive: true });
const fixture = fs.mkdtempSync(path.join(root, 'target/lint-policy.'));
const tables = fs.readFileSync('Cargo.toml', 'utf8').split('[workspace.lints.cargo]')[1].split('[profile.')[0];
fs.writeFileSync(path.join(fixture, 'Cargo.toml'), `[workspace]\nmembers = ["probe"]\nresolver = "3"\n[workspace.lints.cargo]${tables}`);
fs.mkdirSync(path.join(fixture, 'probe'));
fs.writeFileSync(path.join(fixture, 'probe/Cargo.toml'), `[package]\nname = "wire-lint-probe"\nversion = "0.0.0"\nedition = "2024"\nrust-version = "${floor}"\n[lib]\npath = "lib.rs"\n[lints]\nworkspace = true\n`);
for (const [source, diagnostic] of [
  ['pub fn probe() { Err::<(), ()>(()); }', '-D unused-must-use'],
  ['pub unsafe fn probe() {}', 'declaration of an `unsafe` function'],
]) {
  fs.writeFileSync(path.join(fixture, 'probe/lib.rs'), `//! Deliberately rejected policy fixture.\n/// Probe.\n${source}\n`);
  const result = spawnSync('cargo', ['check', '--offline', '--manifest-path', path.join(fixture, 'Cargo.toml'), '--target-dir', path.join(fixture, 'build')], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.notEqual(result.status, 0);
  assert.ok(result.stderr.includes(diagnostic), result.stderr);
}
// Exercise the public adapter's warning mode without compiler-level denial.
// A Cargo warning must fail both compilation and replay of the same artifacts.
fs.writeFileSync(path.join(fixture, 'probe/lib.rs'), `//! Warning-mode probe.
/// Probe.
#[must_use]
pub fn probe() -> u32 { return 7; }
`);
const warningArgs = ['clippy', '--offline', '--manifest-path', path.join(fixture, 'Cargo.toml'), '--target-dir', path.join(fixture, 'warnings'), '--verbose'];
for (const warm of [false, true]) {
  const result = spawnSync('cargo', warningArgs, {
    encoding: 'utf8', env: { ...process.env, CARGO_BUILD_WARNINGS: 'deny' },
  });
  assert.ifError(result.error);
  assert.notEqual(result.status, 0, 'Cargo accepted a Clippy warning');
  assert.match(result.stderr, /needless_return|unneeded `return`/);
  if (warm) assert.match(result.stderr, /Fresh wire-lint-probe/, result.stderr);
}
fs.writeFileSync(path.join(fixture, 'probe/lib.rs'), '//! Passing control.\n/// Probe.\n#[must_use]\npub fn probe() -> u32 { 7 }\n');
const clean = spawnSync('cargo', warningArgs, {
  encoding: 'utf8', env: { ...process.env, CARGO_BUILD_WARNINGS: 'deny' },
});
assert.ifError(clean.error);
assert.equal(clean.status, 0, clean.stderr);
console.log('Rust policy passed: inheritance, must-use/unsafe rejection and fresh/cached Clippy warning refusal.');

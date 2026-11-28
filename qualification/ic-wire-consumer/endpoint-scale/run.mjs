// Run from the repository root. Each input is built using its own qualification lock.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { cases, guest, verify } from './fixture.mjs';

const { values } = parseArgs({
  options: {
    base: { type: 'string' }, candidate: { type: 'string', default: 'WORKTREE' },
    endpoints: { type: 'string', default: '300' }, shape: { type: 'string', default: 'layouts' },
    speed: { type: 'boolean' }, instructions: { type: 'boolean' }, help: { type: 'boolean' },
  }
});
if (values.help) {
  console.log('node qualification/ic-wire-consumer/endpoint-scale/run.mjs [--base REF] [--candidate REF|WORKTREE] [--endpoints 1..1000] [--shape layouts|ids] [--speed] [--instructions]');
  process.exit(0);
}
const count = Number(values.endpoints), shape = values.shape;
assert(Number.isInteger(count) && count >= 1 && count <= 1000, 'use 1 to 1000 endpoints');
assert(['layouts', 'ids'].includes(shape), 'shape must be layouts or ids');
if (values.instructions) assert(process.env.POCKET_IC_BIN, 'set POCKET_IC_BIN to a prepared PocketIC executable');
for (const [key, value] of Object.entries(process.env)) {
  assert(!value || !/^(RUSTFLAGS$|CARGO_ENCODED_RUSTFLAGS$|RUSTC(_WORKSPACE)?_WRAPPER$|CARGO_PROFILE_|CARGO_TARGET_.*_RUSTFLAGS$)/.test(key), 'unset ' + key + ' for this measurement');
}

function run(tool, args, options = {}) {
  const r = spawnSync(tool, args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, ...options });
  if (r.error) throw r.error;
  if (r.status !== 0) throw Error(`${tool} ${args.join(' ')} failed\n${r.stderr ?? ''}`);
  return r.stdout?.trimEnd();
}
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const flags = ['mutable-globals', 'nontrapping-float-to-int', 'bulk-memory', 'sign-ext', 'reference-types',
  'multivalue', 'bulk-memory-opt', 'call-indirect-overlong'].map(f => '--enable-' + f);
const cache = resolve('qualification/target/endpoint-scale');
mkdirSync(cache, { recursive: true });
const output = mkdtempSync(join(cache, 'run-'));
console.log('Endpoint scale report:', output);
const report = { count, shape, rustc: run('rustc', ['-Vv']), binaryen: run('wasm-opt', ['--version']), flags, builds: [] };

// Copy only Rust workspace inputs. Each revision supplies its own manifests and locks.
const paths = ['Cargo.toml', 'Cargo.lock', 'crates', 'qualification/Cargo.toml', 'qualification/Cargo.lock',
  ...['ic-wire-consumer', 'ic-wire-instructions', 'ic-wire-compatibility'].map(s => 'qualification/' + s)];
function snapshot(ref) {
  const stage = mkdtempSync(join(cache, 'source-'));
  const working = ref === 'WORKTREE';
  const commit = run('git', ['rev-parse', '--verify', '--end-of-options', (working ? 'HEAD' : ref) + '^{commit}']);
  if (working) {
    const files = run('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...paths]);
    for (const file of files.split('\0').filter(Boolean)) {
      if (!existsSync(file) || file.startsWith('qualification/ic-wire-consumer/endpoint-scale/')) continue;
      mkdirSync(dirname(join(stage, file)), { recursive: true });
      cpSync(file, join(stage, file));
    }
  } else {
    const archive = join(stage, 'input.tar');
    run('git', ['archive', '--format=tar', '-o', archive, commit, '--', ...paths]);
    run('tar', ['-xf', archive, '-C', stage]);
    rmSync(archive);
  }
  const consumer = join(stage, 'qualification/ic-wire-consumer');
  rmSync(join(consumer, 'endpoint-scale'), { recursive: true, force: true });
  const legacy = /^evolution\s*=/m.test(readFileSync(join(stage, 'crates/ic-wire/Cargo.toml'), 'utf8'));
  // Wire 0.32 switched derives to explicit per-type sparse layout selection.
  const manifest = readFileSync(join(stage, 'Cargo.toml'), 'utf8');
  const version = manifest.match(/\[workspace\.package\][^[]*?^version\s*=\s*"(\d+)\.(\d+)\./m);
  assert(version, 'missing Wire workspace version');
  const explicitSparse = Number(version[1]) === 0 && Number(version[2]) === 32;
  const src = guest(count, shape, legacy, false, explicitSparse);
  for (const [name, code] of [['endpoint-scale', src], ['endpoint-scale-counter', guest(count, shape, legacy, true, explicitSparse)]]) {
    const path = 'examples/' + name.replaceAll('-', '_') + '.rs';
    writeFileSync(join(consumer, path), code);
    writeFileSync(join(consumer, 'Cargo.toml'), `\n[[example]]\nname="${name}"\npath="${path}"\ncrate-type=["cdylib"]\n`, { flag: 'a' });
  }
  const digest = createHash('sha256');
  function hashTree(dir) {
    for (const name of readdirSync(join(stage, dir)).sort()) {
      const relative = join(dir, name), path = join(stage, relative);
      if (statSync(path).isDirectory()) hashTree(relative);
      else digest.update(relative + '\0' + sha(readFileSync(path)));
    }
  }
  hashTree('');
  const source = digest.digest('hex'), dir = join(cache, source);
  try { renameSync(stage, dir); }
  catch (err) {
    if (!['ENOTEMPTY', 'EEXIST'].includes(err.code)) throw err;
    rmSync(stage, { recursive: true });
  }
  return { dir, ref, commit, source, guest_sha256: sha(src), lock_sha256: sha(readFileSync(join(dir, 'qualification/Cargo.lock'))) };
}
function codeBytes(bytes) {
  let at = 8;
  while (at < bytes.length) {
    const id = bytes[at++];
    let size = 0, shift = 0, b;
    do { b = bytes[at++]; size += (b & 127) * 2 ** shift; shift += 7; } while (b & 128);
    if (id === 10) return size;
    at += size;
  }
  throw Error('missing Wasm code section');
}
function build(input, profile, name = 'endpoint-scale') {
  const target = join(input.dir, 'target');
  const manifest = readFileSync(join(input.dir, 'crates/ic-wire/Cargo.toml'), 'utf8');
  const features = /^derive\s*=/m.test(manifest) ? 'ic,wire/derive' : 'ic';
  run('cargo', ['build', '--locked', '--manifest-path', join(input.dir, 'qualification/Cargo.toml'),
    '-p', 'ic-wire-consumer', '--example', name, '--features', features,
    '--target', 'wasm32-unknown-unknown', '--target-dir', target, '--profile', profile]);
  return join(target, 'wasm32-unknown-unknown', profile, 'examples', name.replaceAll('-', '_') + '.wasm');
}
for (const [label, ref] of [['base', values.base], ['candidate', values.candidate]]) {
  if (!ref) continue;
  const input = snapshot(ref);
  const { ref: selected, commit, source, guest_sha256, lock_sha256 } = input;
  const row = { label, ref: selected, commit, source, guest_sha256, lock_sha256, artifacts: {} };
  for (const profile of values.speed ? ['wire-size', 'wire-speed'] : ['wire-size']) {
    const raw = build(input, profile);
    const files = { [profile]: join(output, label + '-' + profile + '.wasm') };
    cpSync(raw, files[profile]);
    if (profile === 'wire-size') {
      files.oz = join(output, label + '-oz.wasm');
      run('wasm-opt', ['-Oz', ...flags, raw, '-o', files.oz]);
    }
    for (const [name, path] of Object.entries(files)) {
      const bytes = readFileSync(path);
      verify(bytes, count, shape);
      row.artifacts[name] = { bytes: bytes.length, code_bytes: codeBytes(bytes), sha256: sha(bytes), path };
    }
  }
  if (values.instructions) {
    const path = join(output, label);
    mkdirSync(path);
    run('wasm-opt', ['-Oz', ...flags, build(input, 'wire-size', 'endpoint-scale-counter'), '-o', join(path, 'metered.wasm')]);
    const probes = cases(Math.floor(count / 2), shape).filter(t => ['absent', 'present', 'unknown'].includes(t.kind));
    writeFileSync(join(path, 'cases.json'), JSON.stringify(probes));
    run('cargo', ['run', '--locked', '--manifest-path', 'qualification/Cargo.toml', '-p', 'ic-wire-instructions', '--', path, '--endpoint-scale']);
    row.instructions = JSON.parse(readFileSync(join(path, 'instructions.json'))).rows;
  }
  report.builds.push(row);
}
const [first, second] = report.builds;
const last = second ?? first;
const rows = Object.entries(last.artifacts).map(([profile, a]) => {
  const b = first.artifacts[profile];
  return second ? {
    profile, base_bytes: b.bytes, candidate_bytes: a.bytes, delta_bytes: a.bytes - b.bytes,
    delta_percent: Number((100 * (a.bytes / b.bytes - 1)).toFixed(2)), code_delta: a.code_bytes - b.code_bytes
  }
    : { profile, bytes: a.bytes, code_bytes: a.code_bytes };
});
report.sizes = rows;
if (values.instructions) {
  report.instruction_samples = last.instructions.map((sample, i) => {
    const { method, sample: repeat, instructions } = sample;
    const row = { method, case: sample.case, sample: repeat, instructions };
    if (second) row.delta_instructions = instructions - first.instructions[i].instructions;
    return row;
  });
}
writeFileSync(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
console.table(rows);
if (values.instructions) console.table(report.instruction_samples);

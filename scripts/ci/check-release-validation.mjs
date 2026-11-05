// Consumer proof: actual selector/Shared runner/receipt, disposable Git effects only.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
fs.mkdirSync(path.join(root, 'target'), { recursive: true });
const directory = fs.mkdtempSync(path.join(root, 'target/release-validation-fixture.'));
const repo = path.join(directory, 'repo'); fs.mkdirSync(repo);
const env = { ...process.env };
for (const key of Object.keys(env)) if (/^GIT_/.test(key) || ['MAKEFLAGS', 'MFLAGS', 'MAKEOVERRIDES', 'GNUMAKEFLAGS', 'MAKEFILES', 'RELEASE_COMMIT', 'VALIDATION_INPUT_ROOT', 'VALIDATION_INPUT_BASELINE'].includes(key)) delete env[key];
const exec = (file, args, extra = {}, expected = 0) => {
  const r = spawnSync(file, args, { cwd: repo, env: { ...env, ...extra }, encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 });
  assert.ifError(r.error);
  if (expected === 0) assert.equal(r.status, 0, `${file} ${args.join(' ')}\n${r.stdout}\n${r.stderr}`);
  else assert.notEqual(r.status, 0, `${file} unexpectedly passed`);
  return r;
};
const git = (...args) => exec('git', args).stdout.trim();
const put = (p, value) => { fs.mkdirSync(path.dirname(path.join(repo, p)), { recursive: true }); fs.writeFileSync(path.join(repo, p), value); };
const copy = p => put(p, fs.readFileSync(path.join(root, p)));
for (const p of ['ci/release-validation.json', 'ci/cargo-release-groups.json', 'scripts/ci/wire-release-validation.mjs',
  'scripts/ci/check-release-source.sh', 'scripts/ci/run-validation-targets.sh', 'scripts/ci/check-make-execution.sh',
  'scripts/ci/verify-file-checksum.sh', 'scripts/ci/validation-input-checkpoint.pl', 'scripts/ci/release-validation-plan.pl',
  'scripts/ci/ValidationPlan.pm', 'scripts/ci/validation-evidence.pl', 'scripts/ci/wire-validation-roster.pl']) copy(p);
const stages = JSON.parse(fs.readFileSync(path.join(repo, 'ci/release-validation.json'))).stages;
const actual = execFileSync('make', ['--no-print-directory', '-s', 'validation-targets'], { cwd: root, encoding: 'utf8' }).trim().split(/\s+/);
assert.deepEqual(stages, actual);
put('.gitignore', '/target/\n');
put('Makefile', `IC_WIRE_CI_TARGETS := $(shell perl scripts/ci/wire-validation-roster.pl || printf '%s\\n' WIRE_ROSTER_REFUSED)\nifneq ($(filter WIRE_ROSTER_REFUSED,$(IC_WIRE_CI_TARGETS)),)\n$(error Fixture validation roster refused)\nendif\n.PHONY: validation-targets $(IC_WIRE_CI_TARGETS)\nvalidation-targets:\n\t@printf '%s\\n' $(IC_WIRE_CI_TARGETS)\n$(IC_WIRE_CI_TARGETS):\n\t@test ! -f .git/fail-$@\n\t@printf '%s\\n' '$@' >> .git/commands\n\t@if test -f .git/drift; then echo drift >> crates/ic-wire-endpoint/src/lib.rs; fi\n`);
put('Cargo.toml', '[workspace]\n'); put('Cargo.lock', '# fixture\n');
put('crates/ic-wire/src/lib.rs', '// Core fixture\n');
put('crates/ic-wire-endpoint/src/lib.rs', '// Endpoint fixture\n');
git('init', '-q', '-b', 'main'); git('config', 'user.name', 'Fixture'); git('config', 'user.email', 'fixture@example.invalid');
git('config', 'core.hooksPath', '/dev/null'); git('config', 'commit.gpgsign', 'false');
const commit = () => { git('add', '.'); git('commit', '-qm', 'Disposable validation input'); return git('rev-parse', 'HEAD'); };
const baseline = commit(); git('tag', '-a', 'endpoint-v0.5.0', '-m', 'Disposable qualified baseline');
put('crates/ic-wire-endpoint/src/lib.rs', '// Endpoint-only candidate\n'); const source = commit();
const select = (name, group = 'endpoint', extra = {}, expected = 0) => {
  const target = `target/${name}`;
  exec(process.execPath, ['scripts/ci/wire-release-validation.mjs', 'select', group, git('rev-parse', 'HEAD'), target], { RELEASE_PREVIOUS: '0.5.0', ...extra }, expected);
  return expected === 0 ? { target, plan: JSON.parse(fs.readFileSync(path.join(repo, target, 'plan.json'))) } : null;
};
const run = (selected, expected = 0) => exec('bash', ['scripts/ci/run-validation-targets.sh', '--plan', `${selected.target}/plan.json`],
  { RELEASE_SOURCE: git('rev-parse', 'HEAD'), RELEASE_GROUP: selected.plan.group, VALIDATION_LOG_DIR: path.join(repo, selected.target, 'logs') }, expected);
const seal = (selected, name, expected = 0) => exec(process.execPath, ['scripts/ci/wire-release-validation.mjs', 'seal', selected.plan.group,
  git('rev-parse', 'HEAD'), selected.target, `.git/${name}`], {}, expected);
const check = (name, group = 'endpoint', expected = 0) => exec(process.execPath, ['scripts/ci/wire-release-validation.mjs', 'check', group, source, `.git/${name}`], {}, expected);
let narrow = select('narrow');
// Hosts/configurations not yet qualified retain full selection.
const configRoots = [env.CARGO_HOME || path.join(process.env.HOME, '.cargo')];
for (let p = repo;; p = path.dirname(p)) {
  configRoots.push(path.join(p, '.cargo')); if (path.dirname(p) === p) break;
}
const qualified = process.platform === 'linux' && process.arch === 'x64' && process.version === 'v24.21.0'
  && Object.entries(env).filter(([k]) => /^(RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTUP_TOOLCHAIN|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_|CARGO_PROFILE_|CARGO_TARGET_[A-Z0-9_]+_(RUSTFLAGS|LINKER|RUNNER)|BASH_ENV|MAKEFILES)/.test(k)).every(([k, v]) => k === 'RUSTUP_TOOLCHAIN' && v === '1.99.1')
  && !configRoots.some(p => ['config', 'config.toml'].some(f => fs.existsSync(path.join(p, f))));
assert.equal(narrow.plan.scope, qualified ? 'group' : 'workspace');
assert.equal(narrow.plan.stages.filter(s => !s.required).length, qualified ? 9 : 0);
// Endpoint-owned generators retain the same narrow consumer obligations.
put('crates/ic-wire-endpoint-macros/src/lib.rs', '// Endpoint macro-only candidate\n');
commit();
assert.equal(select('endpoint-macros').plan.scope, qualified ? 'group' : 'workspace');
git('reset', '--hard', source);
const core = select('core', 'core'); assert(core.plan.stages.every(s => s.required));
run(core); seal(core, 'core-proof');
run(narrow); seal(narrow, 'endpoint-proof'); check('endpoint-proof');
assert.deepEqual(fs.readFileSync(path.join(repo, '.git/commands'), 'utf8').trim().split('\n'),
  [...stages, ...narrow.plan.stages.filter(s => s.required).map(s => s.target)]);
check('endpoint-proof', 'core', 1);
// Required failure and interruption/missing evidence cannot seal or prepare.
put('.git/fail-check-endpoint', 'fail'); const fail = select('required-failure'); run(fail, 1); seal(fail, 'bad-proof', 1); fs.unlinkSync(path.join(repo, '.git/fail-check-endpoint'));
const missingLog = select('missing-log'); run(missingLog);
const missingRun = fs.readdirSync(path.join(repo, missingLog.target, 'logs'))[0];
fs.unlinkSync(path.join(repo, missingLog.target, 'logs', missingRun, '0.log')); seal(missingLog, 'missing-log-proof', 1);
const interrupted = select('interrupted'); seal(interrupted, 'interrupted-proof', 1);
// An excluded codec failure does not block Endpoint, but must block full Core.
put('.git/fail-check-benchmarks', 'fail'); const excluded = select('excluded-failure'); run(excluded, qualified ? 0 : 1);
if (qualified) seal(excluded, 'excluded-proof'); const failedCore = select('core-failure', 'core'); run(failedCore, 1); fs.unlinkSync(path.join(repo, '.git/fail-check-benchmarks'));
// Checkpoint drift must stop subsequent commands, even when a target exits zero.
const drift = select('input-drift'); put('.git/drift', 'drift'); run(drift, 1); fs.unlinkSync(path.join(repo, '.git/drift')); git('reset', '--hard', source);
// Copied proofs still validate after release metadata changes HEAD.
fs.renameSync(path.join(repo, narrow.target), path.join(repo, narrow.target + '.retained'));
check('endpoint-proof');
put('Cargo.toml', '[workspace]\n# prepared release metadata\n'); const prepared = commit(); check('endpoint-proof'); git('reset', '--hard', source);
fs.renameSync(path.join(repo, narrow.target + '.retained'), path.join(repo, narrow.target));
// Replacing raw evidence or losing its original execution must reject recovery.
const proof = path.join(repo, '.git/endpoint-proof'), log = fs.readdirSync(path.join(proof, 'logs'))[0];
const events = path.join(proof, 'logs', log, 'plan-events.tsv'), originalEvents = fs.readFileSync(events);
fs.appendFileSync(events, 'check-endpoint\tFAIL\ttampered\n'); check('endpoint-proof', 'endpoint', 1); fs.writeFileSync(events, originalEvents); check('endpoint-proof');
const sharedProof = path.join(proof, 'shared-evidence.json'), originalProof = fs.readFileSync(sharedProof);
fs.appendFileSync(sharedProof, ' '); check('endpoint-proof', 'endpoint', 1); fs.writeFileSync(sharedProof, originalProof);
const sealPath = path.join(proof, 'seal.json'), originalSeal = fs.readFileSync(sealPath), oldSeal = JSON.parse(originalSeal);
oldSeal.schema = 1; fs.writeFileSync(sealPath, JSON.stringify(oldSeal)+'\n'); check('endpoint-proof', 'endpoint', 1); fs.writeFileSync(sealPath, originalSeal);
check('endpoint-proof');
git('tag', '-fa', 'endpoint-v0.5.0', baseline, '-m', 'Changed baseline object'); check('endpoint-proof', 'endpoint', 1);
// A baseline/schema/unknown stage/input change cannot silently retain narrow scope.
const restoreTag = () => { git('update-ref', 'refs/tags/endpoint-v0.5.0', JSON.parse(fs.readFileSync(path.join(repo, narrow.target, 'context.json'))).tagObject); };
restoreTag();
git('update-ref', '-d', 'refs/tags/endpoint-v0.5.0'); assert.equal(select('missing-baseline').plan.scope, 'workspace'); restoreTag();
git('update-ref', 'refs/tags/endpoint-v0.5.0', source); assert.equal(select('lightweight-baseline').plan.scope, 'workspace'); restoreTag();
for (const [name, file] of [['lock', 'Cargo.lock'], ['core-source', 'crates/ic-wire/src/lib.rs'], ['unknown', 'unknown.input']]) {
  put(file, '// changed shared input\n'); commit(); assert.equal(select(name).plan.scope, 'workspace'); git('reset', '--hard', source);
}
git('update-index', '--assume-unchanged', 'crates/ic-wire-endpoint/src/lib.rs');
put('crates/ic-wire-endpoint/src/lib.rs', '// hidden drift\n'); select('hidden-drift', 'endpoint', {}, 1);
git('update-index', '--no-assume-unchanged', 'crates/ic-wire-endpoint/src/lib.rs'); git('reset', '--hard', source);
assert.equal(select('flags', 'endpoint', { RUSTFLAGS: '--cfg unrelated' }).plan.scope, 'workspace');
select('unknown-group', 'unknown', {}, 1);
const missing = select('missing-stage'); missing.plan.stages.pop(); put(`${missing.target}/plan.json`, JSON.stringify(missing.plan)+'\n'); run(missing, 1);
const policy = JSON.parse(fs.readFileSync(path.join(repo, 'ci/release-validation.json'))); policy.stages.push('new-unclassified-stage'); put('ci/release-validation.json', JSON.stringify(policy)+'\n'); commit();
const expanded = select('new-roster'); assert.equal(expanded.plan.scope, 'workspace'); assert(expanded.plan.stages.every(s => s.required)); assert.equal(expanded.plan.stages.length, stages.length+1);
assert.equal(exec('make', ['--no-print-directory', '-s', 'validation-targets']).stdout.trim().split(/\s+/).length, stages.length+1); git('reset', '--hard', source);
const groups = JSON.parse(fs.readFileSync(path.join(repo, 'ci/cargo-release-groups.json'))); groups.groups = groups.groups.filter(g => g.name !== 'endpoint'); put('ci/cargo-release-groups.json', JSON.stringify(groups)+'\n'); commit();
exec('make', ['--no-print-directory', '-s', 'validation-targets'], {}, 1); select('group-drift', 'endpoint', {}, 1); git('reset', '--hard', source);
const stale = select('stale-source'); put('crates/ic-wire-endpoint/src/lib.rs', '// new source\n'); commit(); run(stale, 1); git('reset', '--hard', source);
fs.writeFileSync(path.join(directory, 'summary.json'), JSON.stringify({ source, baseline, prepared, host: `${process.platform}-${process.arch}`,
  fullStages: stages.length, endpointStages: narrow.plan.stages.filter(s => s.required).length, qualifiedNarrowHost: qualified,
  protections: ['roster/groups', 'shared expansion', 'required/excluded failure', 'source/input drift', 'missing/interrupted evidence', 'proof tampering', 'baseline drift', 'metadata recovery', 'shared verifier/roster/baseline', 'copied proof without original runner', 'old receipt refusal'], noRealReleaseEffects: true }, null, 2)+'\n');
console.log(`Release group validation passed: ${stages.length} full / ${narrow.plan.stages.filter(s => s.required).length} Endpoint stages; evidence ${directory}`);

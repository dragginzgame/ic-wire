// Consumer proof: execute the actual workflow shell steps in disposable Git history.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../../', import.meta.url));
const yq = path.join(root, '.tools/host/bin/yq');
const environment = { ...process.env };
for (const key of Object.keys(environment)) {
  if (key.startsWith('GIT_') || ['MAKEFLAGS', 'MFLAGS', 'MAKEOVERRIDES', 'GNUMAKEFLAGS', 'MAKEFILES'].includes(key)) delete environment[key];
}
function run(binary, args, cwd = root, env = {}, status = 0) {
  const result = spawnSync(binary, args, { cwd, env: { ...environment, ...env }, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, status, `${binary} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
}
const workflow = JSON.parse(run(yq, ['-p', 'yaml', '-o', 'json', '.', '.github/workflows/ci.yml']));
const jobs = workflow.jobs;
assert.equal(workflow.defaults.run.shell, '/bin/bash --noprofile --norc -e -o pipefail {0}');
const hosts = ['ubuntu-24.04', 'macos-15', 'macos-15-intel'];
assert.deepEqual(Object.keys(workflow.on), ['pull_request', 'push']);
assert.equal(workflow.on.pull_request, null);
assert.deepEqual(workflow.on.push, { branches: ['main'] });
assert.equal(jobs.validate.name, '${{ matrix.suite }} (${{ matrix.os }})');
assert.deepEqual(jobs.validate.strategy.matrix, { os: hosts, suite: ['core', 'typescript', 'benchmarks'] });
assert.equal(jobs.validate.strategy['fail-fast'], false);
assert.equal(jobs.validate.needs, 'change-scope');
assert.equal(jobs.validate['runs-on'], '${{ matrix.os }}');
assert.equal(jobs.validate.if, "${{ needs.change-scope.result == 'success' && needs.change-scope.outputs.scope != 'docs' }}");
assert.equal(jobs.docs.needs, 'change-scope');
assert.equal(jobs.docs.if, "${{ needs.change-scope.result == 'success' && needs.change-scope.outputs.scope == 'docs' }}");
assert.equal(jobs.qualification.if, '${{ always() }}');
assert.deepEqual(jobs.qualification.needs, ['change-scope', 'docs', 'validate']);
assert.equal(jobs.qualification.strategy, undefined);
assert.equal(jobs.scaffold, undefined);
assert.equal(jobs.qualification['runs-on'], 'ubuntu-24.04');
assert.equal(jobs.qualification.name, undefined);
assert.equal(jobs['change-scope'].outputs.scope, '${{ steps.classify.outputs.scope }}');
for (const job of Object.values(jobs)) {
  assert.ok(!job['continue-on-error']);
  for (const step of job.steps) assert.ok(!step['continue-on-error']);
}
const classifier = jobs['change-scope'].steps.find(s => s.id === 'classify');
assert.deepEqual(classifier.env, {
  CHANGE_EVENT: '${{ github.event_name }}',
  CHANGE_REF: "${{ github.event_name == 'pull_request' && format('refs/heads/{0}', github.base_ref) || github.ref }}",
  CHANGE_BASE: "${{ github.event_name == 'pull_request' && github.event.pull_request.base.sha || github.event.before }}",
  CHANGE_HEAD: '${{ github.sha }}',
});
assert.equal(jobs['change-scope'].steps[0].with['fetch-depth'], 0);
assert.equal(jobs['change-scope'].steps[0].with['persist-credentials'], false);
const snapshotCheck = jobs['change-scope'].steps.find(s => s.name === 'Check reviewed snapshot integrity');
assert.equal(snapshotCheck.run, 'bash scripts/ci/verify-toko-tooling-snapshot.sh --consumer "$PWD" --require-annotations');
const scan = jobs['change-scope'].steps.find(s => s.name === 'Scan repository history for secrets');
assert.equal(scan.env.GITLEAKS_BIN, '${{ steps.security-tool.outputs.gitleaks }}');
assert.match(scan.run, /"\$GITLEAKS_BIN" git --redact=100/);
assert.match(jobs['change-scope'].steps.find(s => s.id === 'security-tool').run, /scripts\/ci\/install-ci-tool\.sh gitleaks/);
const docs = jobs.docs.steps.find(s => s.name === 'Check prose documentation').run;
assert.equal(docs, 'make check-doc-links');
for (const job of [jobs.docs, jobs['change-scope'], jobs.qualification]) {
  for (const step of job.steps) assert.equal(step.if, undefined, step.name);
}
const native = jobs.validate.steps.find(s => s.name === 'Validate selected CI group');
assert.equal(native.if, undefined);
assert.equal(native.env.CI_GROUP, '${{ matrix.suite }}');
assert.equal(native.run, 'make "ci-$CI_GROUP"');
// Offline pin/boundary admission reads both independent workspaces, including
// in the Core job. A root-only fetch leaves clean runners missing the oracle.
const dependencies = jobs.validate.steps.find(s => s.name === 'Prepare locked workspace dependencies');
assert.equal(dependencies.if, undefined);
assert.deepEqual(dependencies.run.trim().split('\n'), [
  'cargo fetch --locked',
  'cargo fetch --locked --manifest-path qualification/Cargo.toml',
]);
assert.ok(jobs.validate.steps.indexOf(dependencies) < jobs.validate.steps.indexOf(native));
assert.ok(jobs.validate.steps.findIndex(s => s.name === 'Prepare the compiler from local policy') < jobs.validate.steps.indexOf(native));
assert.match(fs.readFileSync(path.join(root, 'Makefile'), 'utf8'), /^ci-core ci-typescript ci-benchmarks .*: toko-tooling-check$/m);
const retention = jobs.validate.steps.find(s => s.uses === './.github/actions/retain-failure-evidence');
assert.equal(retention.if, '${{ failure() }}');
assert.ok(retention.with.name.includes('matrix.suite') && retention.with.name.includes('matrix.os'));
const gate = jobs.qualification.steps[0];
assert.deepEqual(gate.env, {
  CLASSIFICATION_RESULT: '${{ needs.change-scope.result }}',
  CHANGE_SCOPE: '${{ needs.change-scope.outputs.scope }}',
  DOCS_RESULT: '${{ needs.docs.result }}',
  VALIDATION_RESULT: '${{ needs.validate.result }}',
});

const policyPath = 'ci/docs-only-paths.txt';
const policy = fs.readFileSync(path.join(root, policyPath), 'utf8');
const allowed = policy.split('\n').filter(p => p && !p.startsWith('#'));
assert.deepEqual(allowed, ['docs/guides/design.md', 'docs/benchmarks/evaluation.md', 'docs/benchmarks/performance.md', 'docs/status/current.md']);
const snapshot = fs.readFileSync(path.join(root, '.toko-tooling.snapshot'), 'utf8');
for (const p of allowed) {
  const info = fs.lstatSync(path.join(root, p));
  assert.ok(info.isFile() && !(info.mode & 0o111), p);
  assert.ok(!snapshot.split('\n').some(line => line.startsWith('file\t') && line.endsWith('\t' + p)), p);
  assert.ok(!/Shared companions:/.test(fs.readFileSync(path.join(root, p), 'utf8')), p);
}

fs.mkdirSync(path.join(root, 'target'), { recursive: true });
const evidence = fs.mkdtempSync(path.join(root, 'target/ci-selection.'));
const repo = path.join(evidence, 'repo');
fs.mkdirSync(repo);
// Include new candidate inputs before staging, including linked evidence and
// fixtures. A hand-maintained additions list silently tests an incomplete tree.
const files = run('git', ['ls-files', '--cached', '--others', '--exclude-standard', '-z']).split('\0').filter(Boolean);
// Unstaged moves leave deleted index entries until the maintainer stages them.
// Admit those explicit deletions; other read/metadata failures still stop export.
const deleted = new Set(run('git', ['ls-files', '--deleted', '-z']).split('\0').filter(Boolean));
for (const p of new Set(files)) {
  if (deleted.has(p)) continue;
  const source = path.join(root, p), dest = path.join(repo, p);
  assert.ok(fs.lstatSync(source).isFile(), p);
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(source, dest);
  fs.chmodSync(dest, fs.statSync(source).mode & 0o777);
}
const git = (...args) => run('git', args, repo);
// A clean hosted runner has no pinned compiler yet. Execute the real bootstrap
// command with an empty Rustup home, then prove that payload drift still fails.
const emptyRustup = path.join(evidence, 'empty-rustup');
fs.mkdirSync(emptyRustup);
const bootstrapEnv = { RUSTUP_HOME: emptyRustup, RUSTUP_AUTO_INSTALL: '0', RUSTUP_TOOLCHAIN: 'wire-ci-no-installed-toolchain' };
run('bash', ['-e', '-o', 'pipefail', '-c', snapshotCheck.run], repo, bootstrapEnv);
const managedFile = path.join(repo, 'scripts/dev/cloc.sh');
const managedBytes = fs.readFileSync(managedFile);
fs.appendFileSync(managedFile, '\n# Injected snapshot drift\n');
run('bash', ['-e', '-o', 'pipefail', '-c', snapshotCheck.run], repo, bootstrapEnv, 1);
fs.writeFileSync(managedFile, managedBytes);
git('init', '-q', '-b', 'main');
git('config', 'user.name', 'Fixture');
git('config', 'user.email', 'fixture@example.invalid');
git('config', 'core.hooksPath', '/dev/null');
git('config', 'commit.gpgsign', 'false');
git('config', 'core.filemode', 'true');
git('add', '.');
git('commit', '-qm', 'Disposable workflow baseline');
const base = git('rev-parse', 'HEAD');
const modify = p => fs.appendFileSync(path.join(repo, p), '\nFixture prose edit.\n');
let cases = 0;
function classify(label, expected, options = {}, status = 0) {
  const output = path.join(evidence, `output-${cases++}`);
  const env = { CHANGE_EVENT: 'push', CHANGE_REF: 'refs/heads/main', CHANGE_BASE: base, CHANGE_HEAD: git('rev-parse', 'HEAD'), GITHUB_OUTPUT: output, ...options };
  const result = spawnSync('bash', ['--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', classifier.run], { cwd: repo, env: { ...environment, ...env }, encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status === 0, status === 0, `${label}: exit ${result.status}: ${result.stderr}`);
  if (!status) assert.equal(fs.readFileSync(output, 'utf8'), `scope=${expected}\n`, label);
  else assert.ok(!fs.existsSync(output), label);
}
function change(label, paths, expected) {
  git('switch', '-q', '-c', `case-${cases}`, base);
  for (const p of paths) modify(p);
  git('add', ...paths); git('commit', '-qm', label);
  classify(label, expected);
}
change('allowed prose', ['docs/benchmarks/performance.md'], 'docs');
run('bash', ['-e', '-o', 'pipefail', '-c', docs], repo);
fs.appendFileSync(path.join(repo, 'docs/benchmarks/performance.md'), '\n[Broken](missing-ci-selection-target.md)\n');
const broken = spawnSync('bash', ['-e', '-o', 'pipefail', '-c', docs], { cwd: repo, env: environment, encoding: 'utf8' });
assert.ifError(broken.error); assert.notEqual(broken.status, 0, 'real documentation check must reject a broken link');
git('restore', 'docs/benchmarks/performance.md');
const exportedDoc = 'TOKO-TOOLING.md';
fs.appendFileSync(path.join(repo, exportedDoc), '\n[Broken](missing-exported-target.md)\n');
const brokenExport = spawnSync('bash', ['-e', '-o', 'pipefail', '-c', docs], { cwd: repo, env: environment, encoding: 'utf8' });
assert.ifError(brokenExport.error);
assert.notEqual(brokenExport.status, 0, 'real documentation check must reject a broken exported Shared link');
assert.match(brokenExport.stderr, /missing-exported-target\.md/);
git('restore', exportedDoc);
classify('tag ref', 'full', { CHANGE_REF: 'refs/tags/core-v0.14.3' });
classify('missing history', 'full', { CHANGE_BASE: '0'.repeat(40) });
classify('unsupported event', 'full', { CHANGE_EVENT: 'workflow_dispatch' });
classify('wrong tested checkout', '', { CHANGE_HEAD: base }, 1);
const helper = path.join(repo, 'scripts/ci/classify-ci-change.pl');
fs.writeFileSync(helper, 'die "injected classifier failure\\n";\n');
classify('classifier failure', '', {}, 255);
fs.writeFileSync(helper, 'print "unknown\\n";\n');
classify('invalid classifier output', 'full');
git('restore', 'scripts/ci/classify-ci-change.pl');
change('source edit', ['crates/ic-wire/src/lib.rs'], 'full');
change('mixed edit', ['docs/benchmarks/performance.md', 'crates/ic-wire/src/lib.rs'], 'full');
change('policy edit', [policyPath], 'full');
change('workflow edit', ['.github/workflows/ci.yml'], 'full');
change('packaged doctest Markdown', ['crates/ic-wire/README.md'], 'full');
change('shared baseline Markdown', ['TOKO-TOOLING.md'], 'full');
change('unknown Markdown', ['docs/guides/development.md'], 'full');
git('switch', '-q', '-c', 'pr-base', base);
modify('crates/ic-wire/src/lib.rs'); git('commit', '-qam', 'Base-only source edit');
const advancedBase = git('rev-parse', 'HEAD');
git('switch', '-q', '-c', 'pr-prose', base);
modify('docs/guides/design.md'); git('commit', '-qam', 'PR prose');
git('merge', '-q', '--no-edit', 'pr-base');
classify('exact tested PR merge', 'docs', { CHANGE_EVENT: 'pull_request', CHANGE_BASE: advancedBase });
git('switch', '-q', '-c', 'whitespace', base);
fs.appendFileSync(path.join(repo, 'docs/benchmarks/performance.md'), '\nTrailing whitespace. \n');
git('commit', '-qam', 'Invalid prose whitespace');
classify('documentation whitespace failure', '', {}, 1);

let gates = 0;
for (const scope of ['docs', 'full', '', 'unknown']) {
  for (const classification of ['success', 'failure', 'skipped', 'cancelled']) {
    for (const docResult of ['success', 'failure', 'skipped', 'cancelled']) {
      for (const nativeResult of ['success', 'failure', 'skipped', 'cancelled']) {
        const expected = classification === 'success' && ((scope === 'docs' && docResult === 'success' && nativeResult === 'skipped') || (scope === 'full' && nativeResult === 'success' && docResult === 'skipped'));
        const result = spawnSync('bash', ['-e', '-o', 'pipefail', '-c', gate.run], { cwd: repo, env: { ...environment, CHANGE_SCOPE: scope, CLASSIFICATION_RESULT: classification, DOCS_RESULT: docResult, VALIDATION_RESULT: nativeResult }, encoding: 'utf8' });
        assert.ifError(result.error); assert.equal(result.status === 0, expected, JSON.stringify({ scope, classification, docResult, nativeResult }));
        gates++;
      }
    }
  }
}
fs.writeFileSync(path.join(evidence, 'results.json'), JSON.stringify({ classifications: cases, requiredGateCombinations: gates, fullNativeCombinations: 9, docsNativeCombinations: 0, brokenDocumentationRejected: true }, null, 2) + '\n');
console.log(`CI selection passed: ${cases} workflow classifications, ${gates} required-gate combinations, real broken-link refusal. Evidence: ${evidence}`);

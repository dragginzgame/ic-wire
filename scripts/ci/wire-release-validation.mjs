// Wire owns scope and receipts; Shared owns plan admission, checkpoints and dispatch.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fs.realpathSync(path.join(path.dirname(fileURLToPath(import.meta.url)), '../..'));
const policyPath = 'ci/release-validation.json';
const skipped = {
  'check-ci-selection': 'CI workflow and classifier unchanged; repository CI retains every native host',
  'check-tooling': 'Make and hook inputs unchanged; fresh tool, format and dependency admission still run',
  'check-release': 'Release implementation unchanged; fresh source, group, plan and receipt admission still run',
  check: 'Core codec and derive inputs unchanged; adapter consumption compiles in check-endpoint',
  test: 'Core codec unchanged; Endpoint native contracts run in check-endpoint',
  'check-typed-diagnostics': 'Core derive unchanged; Endpoint-specific diagnostics run in check-endpoint',
  'check-qualification': 'Core allocation/serialization unchanged; Endpoint borrowing consumer compiles separately',
  'check-benchmarks': 'Codec corpus unchanged; the affected Endpoint input-borrowing consumer compiles in check-endpoint',
  'check-wasm': 'Default Core unchanged; Endpoint and its Core consumption build for Wasm in check-endpoint',
};
const canonical = value => JSON.stringify(value, (_, v) => v && typeof v === 'object' && !Array.isArray(v)
  ? Object.fromEntries(Object.keys(v).sort().map(k => [k, v[k]])) : v) + '\n';
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const run = (file, args, options = {}) => execFileSync(file, args,
  { cwd: root, maxBuffer: 32 * 1024 * 1024, ...options });
const git = (...args) => run('git', args).toString().trim();
const at = (source, file) => run('git', ['show', `${source}:${file}`]);
const shared = (operation, ...args) => run('perl', ['scripts/ci/validation-evidence.pl', operation, root, ...args], { maxBuffer: 4 * 1024 * 1024 });
function requireThat(ok, message) { if (!ok) throw new Error(message); }
function regular(file, bound = 64 * 1024 * 1024) {
  let p = path.parse(path.resolve(file)).root;
  for (const part of path.resolve(file).slice(p.length).split(path.sep)) {
    p = path.join(p, part);
    requireThat(!fs.lstatSync(p).isSymbolicLink(), `redirected evidence path: ${p}`);
  }
  const stat = fs.statSync(file);
  requireThat(stat.isFile() && stat.size <= bound, `not a bounded regular file: ${file}`);
  return fs.readFileSync(file);
}
function relative(file) {
  const p = path.relative(root, path.resolve(file));
  requireThat(p && !p.startsWith('../') && !path.isAbsolute(p) && !/[\r\n\t\0]/.test(p), 'plan must remain in checkout');
  return p;
}
function inventory(source) {
  const entries = run('git', ['ls-tree', '-rz', '--full-tree', source]).toString().split('\0').filter(Boolean);
  return entries.map(row => {
    const [metadata, file] = row.split('\t');
    requireThat(/^100(?:644|755) blob /.test(metadata) && file && !/[\r\n\t\0]/.test(file), 'source must contain regular, canonical files');
    return file;
  });
}
function admitSource(source) {
  requireThat(git('rev-parse', 'HEAD') === source, 'source moved');
  run('bash', ['scripts/ci/check-release-source.sh']);
  const entries = run('git', ['ls-tree', '-rz', '--full-tree', source]).toString().split('\0').filter(Boolean);
  const files = inventory(source);
  const actual = run('git', ['hash-object', '--no-filters', '--stdin-paths'], { input: files.map(f => './' + f).join('\n') + '\n' }).toString().trim().split('\n');
  requireThat(actual.length === entries.length && entries.every((row, i) => {
    const [mode, , object] = row.split('\t')[0].split(' '), stat = fs.lstatSync(path.join(root, files[i]));
    return stat.isFile() && !stat.isSymbolicLink() && Boolean(stat.mode & 0o111) === (mode === '100755') && object === actual[i];
  }), 'working bytes/modes differ from admitted Git source');
}
function environment() {
  // Capture compiler controls, not credentials. Unknown compiler controls widen scope.
  const controls = Object.keys(process.env).filter(k => /^(RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTUP_TOOLCHAIN|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_|CARGO_PROFILE_|CARGO_TARGET_[A-Z0-9_]+_(RUSTFLAGS|LINKER|RUNNER)|BASH_ENV|MAKEFILES)/.test(k)).sort();
  return { host: `${process.platform}-${process.arch}`, node: process.version, cwd: root,
    cargoHome: process.env.CARGO_HOME || path.join(os.homedir(), '.cargo'), targetDirectory: process.env.CARGO_TARGET_DIR || null,
    make: process.env.RELEASE_MAKE || 'make', path: process.env.PATH,
    controls: Object.fromEntries(controls.map(k => [k, process.env[k]])) };
}
function selection(group, source, previous) {
  requireThat(/^[a-f0-9]{40}(?:[a-f0-9]{24})?$/.test(source) && git('cat-file', '-t', source) === 'commit', 'invalid source');
  requireThat(/^\d+\.\d+\.\d+$/.test(previous), 'invalid preceding version');
  const policyBytes = at(source, policyPath), policy = JSON.parse(policyBytes);
  requireThat(regular(path.join(root, policyPath)).equals(policyBytes), 'selection policy differs from source');
  const roster = run('perl', ['scripts/ci/wire-validation-roster.pl'], { maxBuffer: 4 * 1024 * 1024 }).toString().trim().split(/\s+/);
  requireThat(policy.groups.includes(group) && canonical(roster) === canonical(policy.stages), 'unknown group or changed source roster');
  for (const name of Object.keys(skipped)) requireThat(roster.includes(name), 'unreviewed stage omission');
  const baselineRef = `refs/tags/${group}-v${previous}`;
  const observed = JSON.parse(shared('baseline', source, baselineRef));
  const baseline = observed.baseline ?? null, tagObject = observed.object ?? null, changes = observed.changes;
  const config = environment();
  const controlled = Object.entries(config.controls).every(([k, v]) => k === 'RUSTUP_TOOLCHAIN' && v === '1.99.1');
  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  let parent = root;
  const configurationRoots = [cargoHome];
  while (true) {
    configurationRoots.push(path.join(parent, '.cargo'));
    const next = path.dirname(parent); if (next === parent) break; parent = next;
  }
  const ambientConfig = configurationRoots.some(p => ['config', 'config.toml'].some(f => fs.existsSync(path.join(p, f))));
  const endpointPath = p => /^crates\/ic-wire-endpoint(?:-macros)?\/src\/.+\.rs$/.test(p)
    || /^crates\/ic-wire-endpoint(?:-macros)?\/(README|CHANGELOG)\.md$/.test(p)
    || /^qualification\/ic-wire-endpoint-fixture\/(src|tests)\/.+\.rs$/.test(p)
    || p === 'qualification/ic-wire-endpoint-fixture/README.md' || p === 'docs/endpoints.md' || p === 'docs/status/current.md'
    || p === 'qualification/ic-wire-benchmarks/benches/input_borrowing/mod.rs'
    || /^releases\/endpoint\/[^/]+\.md$/.test(p);
  let reason = 'Core codec/derive identities and all reverse consumers require the complete roster';
  let narrow = false;
  if (group === 'endpoint') {
    reason = !observed.available ? observed.reason
        : config.host !== 'linux-x64' ? 'Narrow native mapping is qualified only on Linux x64; other hosts keep full gate'
          : config.node !== 'v24.21.0' || !controlled || ambientConfig ? 'Nonstandard tool/compiler configuration requires full gate'
            : !changes.every(endpointPath) ? 'Shared, manifest, lock, Core, generator or uncertain inputs changed'
              : 'Endpoint and its explicit reverse consumers only; shared/Core inputs equal the qualified baseline';
    narrow = observed.available && config.host === 'linux-x64'
      && config.node === 'v24.21.0' && controlled && !ambientConfig && changes.every(endpointPath);
  }
  return { policy, context: { schema: 1, source, group, previous, baselineRef, baseline, tagObject, changes, configuration: config },
    scope: narrow ? 'group' : 'workspace', stages: roster.map(target => ({ target,
      required: !(narrow && Object.hasOwn(skipped, target)),
      reason: narrow && Object.hasOwn(skipped, target) ? skipped[target] : reason })) };
}
// Product intent is re-admitted across metadata preparation; Shared owns run semantics.
function checkSelection(directory, group, source) {
  const planBytes = regular(path.join(directory, 'plan.json'), 1024 * 1024), plan = JSON.parse(planBytes);
  const contextBytes = regular(path.join(directory, 'context.json'), 1024 * 1024), context = JSON.parse(contextBytes);
  const contextPath = plan.inputs.at(-1);
  requireThat(typeof contextPath === 'string' && /^target\/[^\r\n\t]+\/context\.json$/.test(contextPath)
    && !contextPath.split('/').includes('..'), 'invalid context input');
  const expected = selection(group, source, context.previous);
  const expectedPlan = { schema: 1, source, group, scope: expected.scope, policy: policyPath,
    inputs: [...inventory(source), contextPath], stages: expected.stages };
  requireThat(canonical(context) === canonical(expected.context) && canonical(context) === contextBytes.toString(),
    'changed baseline, host or selection context');
  requireThat(canonical(expectedPlan) === planBytes.toString(), 'source inputs or product selection changed');
  return { plan, planBytes, contextBytes };
}
function retainedLogs(evidence) {
  const name = path.basename(evidence.run);
  requireThat(/^validation\.[A-Za-z0-9]+$/.test(name), 'invalid retained run identity');
  return Object.entries(evidence.files).map(([file, digest]) => {
    requireThat(path.basename(file) === file && file !== '.' && file !== '..', 'redirected retained log');
    return { file: path.join('logs', name, file), digest };
  });
}
function inspect(directory, group, source) {
  const { plan, planBytes, contextBytes } = checkSelection(directory, group, source);
  const bytes = regular(path.join(directory, 'shared-evidence.json'), 1024 * 1024), evidence = JSON.parse(bytes);
  requireThat(canonical(evidence) === bytes.toString() && evidence.schema === 1 && evidence.kind === 'group-validation-evidence'
    && evidence.source === source && evidence.group === group && evidence.scope === plan.scope, 'wrong shared evidence');
  requireThat(evidence.plan_sha256 === sha(planBytes) && evidence.policy_sha256 === sha(at(source, policyPath))
    && evidence.context_sha256 === sha(contextBytes), 'shared evidence binding changed');
  // Byte custody of the copied proof is local. Never reparse Shared's TSV protocol
  // or run its location-bound verifier on prepared metadata/relocated records.
  for (const { file, digest } of retainedLogs(evidence))
    requireThat(sha(regular(path.join(directory, file))) === digest, `retained evidence changed: ${file}`);
  return { plan, evidence };
}

const [operation, group, source, argument, destination] = process.argv.slice(2);
if (operation === 'select') {
  admitSource(source);
  const previous = process.env.RELEASE_PREVIOUS;
  const { policy, context, scope, stages } = selection(group, source, previous);
  const directory = path.resolve(argument), output = relative(directory);
  requireThat(output.startsWith('target/'), 'plan output must be under target/');
  let cursor = root;
  for (const part of output.split('/')) {
    cursor = path.join(cursor, part);
    if (!fs.existsSync(cursor)) fs.mkdirSync(cursor);
    requireThat(fs.lstatSync(cursor).isDirectory() && !fs.lstatSync(cursor).isSymbolicLink(), 'redirected plan output');
  }
  fs.writeFileSync(path.join(directory, 'context.json'), canonical(context), { flag: 'wx' });
  const plan = { schema: 1, source, group, scope, policy: policyPath,
    inputs: [...inventory(source), relative(path.join(directory, 'context.json'))], stages };
  fs.writeFileSync(path.join(directory, 'plan.json'), canonical(plan), { flag: 'wx' });
  requireThat(policy.stages.length === stages.length, 'unclassified stage');
} else if (operation === 'seal') {
  admitSource(source);
  const directory = path.resolve(argument), { plan } = checkSelection(directory, group, source);
  const runs = fs.readdirSync(path.join(directory, 'logs')).filter(n => n.startsWith('validation.'));
  requireThat(runs.length === 1, 'missing or ambiguous runner evidence');
  const evidenceBytes = shared('verify', relative(path.join(directory, 'plan.json')), source, group,
    relative(path.join(directory, 'logs', runs[0])), relative(path.join(directory, 'context.json')));
  fs.writeFileSync(path.join(directory, 'shared-evidence.json'), evidenceBytes, { flag: 'wx' });
  const { evidence } = inspect(directory, group, source);
  // Internal receipt hard cut: schema 1 parsed events locally. Schema 2 binds a
  // Shared-verified proof; old receipts need a fresh gate, never conversion.
  // This changes release evidence only, not the Wire protocol or retry bytes.
  const seal = { schema: 2, kind: 'wire-group-validation', source, group, scope: plan.scope,
    configuration: environment(), evidence_sha256: sha(evidenceBytes) };
  fs.writeFileSync(path.join(directory, 'seal.json'), canonical(seal), { flag: 'wx' });
  const gitDirectory = fs.realpathSync(git('rev-parse', '--absolute-git-dir'));
  const proofPath = destination && path.relative(gitDirectory, path.resolve(destination));
  requireThat(proofPath && !proofPath.startsWith('../') && !path.isAbsolute(proofPath) && !fs.existsSync(destination), 'invalid proof destination');
  let parent = gitDirectory;
  for (const part of proofPath.split(path.sep).slice(0, -1)) {
    parent = path.join(parent, part);
    requireThat(fs.lstatSync(parent).isDirectory() && !fs.lstatSync(parent).isSymbolicLink(), 'redirected proof destination');
  }
  fs.mkdirSync(destination);
  for (const file of ['plan.json', 'context.json', 'shared-evidence.json', 'seal.json', ...retainedLogs(evidence).map(r => r.file)]) {
    const output = path.join(destination, file);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.copyFileSync(path.join(directory, file), output, fs.constants.COPYFILE_EXCL);
  }
  inspect(destination, group, source);
} else if (operation === 'check') {
  const directory = path.resolve(argument), bytes = regular(path.join(directory, 'seal.json')), seal = JSON.parse(bytes);
  requireThat(canonical(seal) === bytes.toString() && Object.keys(seal).sort().join() === 'configuration,evidence_sha256,group,kind,schema,scope,source'
    && seal.schema === 2 && seal.kind === 'wire-group-validation' && seal.source === source && seal.group === group, 'wrong validation receipt');
  requireThat(canonical(seal.configuration) === canonical(environment()), 'validation configuration changed');
  requireThat(seal.evidence_sha256 === sha(regular(path.join(directory, 'shared-evidence.json'), 1024 * 1024)), 'validation proof changed');
  const { plan } = inspect(directory, group, source);
  requireThat(seal.scope === plan.scope, 'validation scope changed');
  process.stdout.write(canonical({ schema: 1, source, group, proof: sha(bytes) }));
} else throw new Error('usage: wire-release-validation.mjs select|seal|check GROUP SOURCE DIRECTORY [DESTINATION]');

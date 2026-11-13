// Exercise real callers without starting measurements or a PocketIC server.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../../', import.meta.url));
fs.mkdirSync(path.join(root, 'target'), { recursive: true });
const temporary = fs.mkdtempSync(path.join(root, 'target/benchmark-cli-'));
const features = 'wire,candid,ic';
let checks = 0;
function run(binary, args, status = 0, cwd = root, env = process.env) {
  if (binary === 'cargo' && args.includes('ic-wire-benchmarks')) {
    args = [...args.slice(0, 1), '--manifest-path', 'qualification/Cargo.toml', ...args.slice(1)];
  }
  const result = spawnSync(binary, args, { cwd, env, encoding: 'utf8', timeout: 300_000, maxBuffer: 32 * 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, status, `${binary} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  checks++;
  return result;
}
function artifacts(output) {
  return new Map(output.trim().split('\n').filter(Boolean).map(line => JSON.parse(line))
    .filter(row => row.reason === 'compiler-artifact' && row.executable)
    .map(row => [row.target.name, row.executable]));
}
try {
  // Exercise real snapshot creation, stopping at the first measurement command.
  const stub = path.join(temporary, "snapshot-bin");
  fs.mkdirSync(stub);
  fs.writeFileSync(path.join(stub, "cargo"), "#!/bin/sh\nexit 93\n", { mode: 0o755 });
  // Own the executable fixture: /bin/true is not available on every host.
  // A distinct exit code also catches accidentally starting the simulator.
  const pocketIc = path.join(stub, "pocket-ic");
  fs.writeFileSync(pocketIc, "#!/bin/sh\nexit 94\n", { mode: 0o755 });
  for (const script of ["evaluate-codecs.sh", "evaluate-codec-matrix.sh", "evaluate-endpoint-matrix.sh", "evaluate-input-borrowing.sh"]) {
    const output = path.join(temporary, script);
    run("bash", [path.join(root, "scripts/ci", script), output], 93, root, {
      ...process.env, PATH: `${stub}:${process.env.PATH}`, POCKET_IC_BIN: pocketIc,
      CORPUS_FEATURE: "", CODEC_SOURCE_SNAPSHOT: "",
    });
    const source = path.join(output, "source");
    for (const name of ["ic-wire-benchmarks", "ic-wire-qualification", "ic-wire-endpoint-fixture"]) {
      assert.ok(fs.existsSync(path.join(source, "qualification", name, "Cargo.toml")));
    }
    assert.equal(fs.existsSync(path.join(source, "qualification", "qualification")), false);
    for (const name of ["Cargo.toml", "Cargo.lock"]) {
      assert.deepEqual(fs.readFileSync(path.join(source, "qualification", name)),
        fs.readFileSync(path.join(root, "qualification", name)));
    }
  }
  for (const [selection, message] of [
    ['cost-smoke,toko', 'select one benchmark corpus'],
    ['coverage-shapes,contract-shapes', 'select one benchmark corpus'],
    ['bulk,packed-i16', 'select one benchmark corpus'],
    ['derive-scale-1,derive-scale-10', 'select one derive cohort size'],
  ]) {
    const result = run('cargo', ['check', '--locked', '--offline', '-p', 'ic-wire-benchmarks',
      '--example', 'codec-wasm', '--features', `wire,candid,${selection}`], 101);
    assert.ok(result.stderr.includes(message), result.stderr);
  }
  const binaries = artifacts(run('cargo', ['build', '--locked', '-p', 'ic-wire-benchmarks',
    '--features', features, '--bench', 'suite', '--example', 'codec-report',
    '--example', 'codec-instructions', '--example', 'admission-instructions', '--message-format=json']).stdout);
  const output = path.join(temporary, 'must-not-exist');
  for (const [name, binary] of binaries) {
    assert.match(run(binary, ['--help']).stdout, /Usage:/);
    run(binary, [], 2);
    run(binary, ['--unknown'], 2);
    const bad = name === 'suite' ? [output, 'evaluation', '3', '1', 'extra']
      : name === 'admission-instructions' ? ['missing.wasm', output, 'extra']
      : name === 'codec-report' ? [output, 'wire-native', 'extra'] : [output, 'extra'];
    run(binary, bad, 2);
    assert.equal(fs.existsSync(output), false);
  }
  const suite = binaries.get('suite');
  for (const tail of [[], ['bad-profile'], ['evaluation', 'bad'], ['evaluation', '2'],
    ['evaluation', '102'], ['evaluation', '3', 'bad'], ['evaluation', '3', '0'],
    ['evaluation', '3', '101'], ['evaluation', '3', '-1']]) {
    run(suite, [output, ...tail], 2);
    assert.equal(fs.existsSync(output), false);
  }
  // Valid invocations reach create_dir, which must refuse an existing output.
  // This admits defaults/range endpoints without running a timing workload.
  const existing = path.join(temporary, 'existing output');
  fs.mkdirSync(existing);
  fs.writeFileSync(path.join(existing, 'sentinel'), 'retained');
  for (const tail of [['evaluation'], ['benchmark-speed', '101', '100'], ['evaluation', '3', '1', '--bench']]) {
    assert.match(run(suite, [existing, ...tail], 1).stderr, /AlreadyExists/);
  }
  const cargoBench = run('cargo', ['bench', '--locked', '--offline', '--profile', 'dev', '-p',
    'ic-wire-benchmarks', '--features', features, '--bench', 'suite', '--', existing, 'evaluation', '3', '1'], 1);
  assert.match(cargoBench.stderr, /AlreadyExists/);
  assert.equal(fs.readFileSync(path.join(existing, 'sentinel'), 'utf8'), 'retained');
  assert.deepEqual(fs.readdirSync(existing), ['sentinel']);

  const report = binaries.get('codec-report');
  for (const args of [['--artifact'], ['--code-size'], [output, 'unknown'],
    ['--code-size', 'missing.wasm', 'extra'], ['--artifact', 'missing.json', 'extra'],
    ['--artifact', 'missing.json', '--code-size', 'missing.wasm']]) run(report, args, 2);
  const wasm = path.join(temporary, 'empty module.wasm');
  fs.writeFileSync(wasm, Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
  assert.equal(run(report, ['--code-size', wasm]).stdout.trim(), '0');
  const cargoJson = path.join(temporary, 'cargo.json');
  fs.writeFileSync(cargoJson, JSON.stringify({ reason: 'compiler-artifact', target: { name: 'codec-wasm' }, filenames: [wasm] }));
  assert.equal(run(report, ['--artifact', cargoJson]).stdout.trim(), wasm);
  for (const primary of ['wire-native', 'wire-bounded', 'wire-packed-i16']) {
    const directory = path.join(temporary, primary);
    fs.mkdirSync(path.join(directory, 'evaluation-native-1'), { recursive: true });
    const rows = [primary, 'candid'].map(codec => JSON.stringify({ case: 'unit', operation: 'encode-fresh', codec,
      input_sha256: 'same-input', encoded_bytes: 11, measurement: { median_ns: 20 } }));
    fs.writeFileSync(path.join(directory, 'evaluation-native-1', 'samples.jsonl'), rows.join('\n'));
    const args = primary === 'wire-native' ? [directory] : [directory, primary];
    run(report, args);
    const csv = fs.readFileSync(path.join(directory, 'comparison.csv'));
    assert.match(csv.toString(), new RegExp(primary));
    run(report, args, 1);
    assert.deepEqual(fs.readFileSync(path.join(directory, 'comparison.csv')), csv);
  }

  // Qualify the copy-only generator probe in a disposable consumer, inheriting
  // the same host dependency selections as its documented ic-wire example.
  const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--offline', '--no-deps', '--format-version', '1']).stdout);
  const wire = metadata.packages.find(p => p.manifest_path === path.join(root, 'crates/ic-wire/Cargo.toml'));
  const dependency = name => {
    const dep = wire.dependencies.find(d => d.name === name);
    return `${name} = { version = ${JSON.stringify(dep.req)}, default-features = ${dep.uses_default_features}, features = ${JSON.stringify(dep.features)} }`;
  };
  const generator = path.join(temporary, 'generator');
  fs.mkdirSync(path.join(generator, 'src'), { recursive: true });
  fs.copyFileSync(path.join(root, 'scripts/benchmarks/generator/mod.rs'), path.join(generator, 'src/main.rs'));
  fs.copyFileSync(path.join(root, 'Cargo.lock'), path.join(generator, 'Cargo.lock'));
  fs.writeFileSync(path.join(generator, 'Cargo.toml'), `[workspace]\n[package]\nname = "generator-cli-check"\nversion = "0.0.0"\nedition = "2024"\nrust-version = "${wire.rust_version}"\npublish = false\n[dependencies]\nic-wire = { path = ${JSON.stringify(path.join(root, 'crates/ic-wire'))}, features = ["typescript"] }\n${dependency('clap')}\n${dependency('serde_json')}\n`);
  const generatorEnv = { ...process.env, CARGO_TARGET_DIR: metadata.target_directory };
  run('bash', [path.join(root, 'vendor/toko-tooling/scripts/ci/check-cargo-dependency-boundaries.sh'), '--consumer', generator, '--external-ic']);
  const generatorBinary = artifacts(run('cargo', ['build', '--offline', '--message-format=json'], 0, generator, generatorEnv).stdout).get('generator-cli-check');
  assert.match(run(generatorBinary, ['--help']).stdout, /Usage:/);
  run(generatorBinary, [output, 'extra'], 2);
  run(generatorBinary, ['--unknown'], 2);
  assert.equal(fs.existsSync(output), false);
  console.log(`Benchmark CLI: ${checks} focused checks passed; no measurements or servers started.`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

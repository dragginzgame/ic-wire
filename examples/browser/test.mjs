import { createServer } from 'node:http';
import { readFile, writeFile, rm, mkdir, open } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawn, execFileSync } from 'node:child_process';
import { once } from 'node:events';
import { setTimeout as delay } from 'node:timers/promises';
import { resolve } from 'node:path';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { chromium } from 'playwright';

if (!process.env.POCKET_IC_BIN || !process.env.IC_WIRE_BROWSER_WASM) throw new Error('Run make evaluate-browser with a prepared PocketIC server');
await mkdir('artifacts', { recursive: true });
await rm('artifacts/connection.json', { force: true });
const log = await open('artifacts/fixture.log', 'w');
const fixture = spawn(resolve('../../target/debug/examples/browser-fixture'), [process.cwd()], {
  stdio: ['pipe', log.fd, log.fd], env: { ...process.env, IC_WIRE_BROWSER_CALLER: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7)).getPrincipal().toText() },
});
let fixtureError;
fixture.on('error', error => { fixtureError = error; });
const exit = once(fixture, 'exit');
let server;
let browser;
try {
  let connection;
  for (let attempt = 0; attempt < 1200; attempt++) {
    if (fixtureError) throw fixtureError;
    if (fixture.exitCode !== null) throw new Error('Fixture exited; see artifacts/fixture.log');
    try { connection = JSON.parse(await readFile('artifacts/connection.json', 'utf8')); break; }
    catch (error) { if (error.code !== 'ENOENT' && !(error instanceof SyntaxError)) throw error; }
    await delay(50);
  }
  if (!connection) throw new Error('Fixture readiness timeout; see artifacts/fixture.log');
  const vectors = JSON.parse(await readFile('generated/vectors.json', 'utf8'));
  const bundle = await readFile('dist/main.js');
  server = createServer((request, response) => {
    if (request.url === '/main.js') { response.setHeader('Content-Type', 'text/javascript'); response.end(bundle); }
    else { response.setHeader('Content-Type', 'text/html'); response.end('<!doctype html><title>IC Wire browser evaluation</title><h1>Postcard browser evaluation</h1><output>Running…</output>'); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  browser = await chromium.launch();
  const page = await browser.newPage();
  page.on('console', message => console.log(`browser: ${message.text()}`));
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async ({ connection, vectors }) => {
    const { run } = await import('/main.js');
    return run(connection, vectors);
  }, { connection, vectors });
  await page.screenshot({ path: 'artifacts/browser.png' });
  const hashes = {};
  for (const path of ['test.mjs','package.json','package-lock.json','tsconfig.json','dist/main.js','generated/apply.ts','generated/inspect.ts','generated/wire-runtime.ts','generated/vectors.json','src/client.ts','src/main.ts','src/typechecks.ts','../../Cargo.toml','../../Cargo.lock','../../crates/ic-wire/Cargo.toml','../../crates/ic-wire/src/lib.rs','../../crates/ic-wire/src/bounded/mod.rs','../../crates/ic-wire/src/shape/mod.rs','../../crates/ic-wire-canister-evaluation/src/lib.rs','../../crates/ic-wire-canister-evaluation/src/authority/mod.rs','../../crates/ic-wire-canister-evaluation/src/replay/mod.rs','../../crates/ic-wire-canister-evaluation/src/commands/mod.rs','../../crates/ic-wire/src/typescript/mod.rs','../../crates/ic-wire/src/typescript/runtime.ts','../../crates/ic-wire/src/contract/mod.rs','../../crates/ic-wire/src/schema/mod.rs','../../crates/ic-wire-derive/src/lib.rs','../../crates/ic-wire-canister-evaluation/Cargo.toml','../../scripts/ci/evaluate-browser.sh','../../crates/ic-wire-canister-evaluation/examples/browser_fixture/mod.rs',process.env.IC_WIRE_BROWSER_WASM]) {
    hashes[path] = createHash('sha256').update(await readFile(path)).digest('hex');
  }
  hashes['PocketIC binary'] = createHash('sha256').update(await readFile(process.env.POCKET_IC_BIN)).digest('hex');
  const sdk = JSON.parse(await readFile('node_modules/@icp-sdk/core/package.json', 'utf8')).version;
  const report = { ...result, vectors, browser: browser.version(), node: process.version,
    rustc: execFileSync('rustc', ['-vV'], { encoding: 'utf8' }),
    pocketIc: execFileSync(process.env.POCKET_IC_BIN, ['--version'], { encoding: 'utf8' }).trim(),
    sdk, generator: 'ic-wire Wire::SCHEMA', canisterFeatures: ['replay'],
    target: 'wasm32-unknown-unknown', profile: 'canister-evaluation', bundleBytes: bundle.length, hashes };
  await writeFile('artifacts/report.json', JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
} finally {
  await browser?.close();
  if (server) await new Promise(resolve => server.close(resolve));
  fixture.stdin.end('\n');
  const stopped = await Promise.race([exit.then(() => true), delay(5000).then(() => false)]);
  if (!stopped) { fixture.kill('SIGTERM'); await exit; }
  await log.close();
}

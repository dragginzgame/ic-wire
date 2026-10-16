import { createServer } from 'node:http';
import { readFile, writeFile, rm, mkdir, open } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawn, execFileSync } from 'node:child_process';
import { once } from 'node:events';
import { setTimeout as delay } from 'node:timers/promises';
import { resolve } from 'node:path';
import { chromium } from 'playwright';

if (!process.env.POCKET_IC_BIN || !process.env.IC_WIRE_BROWSER_WASM) throw new Error('Run make evaluate-browser with a prepared PocketIC server');
await mkdir('artifacts', { recursive: true });
await rm('artifacts/connection.json', { force: true });
const log = await open('artifacts/fixture.log', 'w');
const fixture = spawn(resolve('../../target/debug/examples/browser-fixture'), [process.cwd()], {
  stdio: ['pipe', log.fd, log.fd], env: process.env,
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
  for (const path of ['package-lock.json','generated/wire-bindings/index.js','generated/wire-bindings/index.d.ts','generated/contract.json','generated/vectors.json','src/client.ts','src/main.ts','../../Cargo.lock','../../crates/ic-wire-canister-evaluation/src/lib.rs','../../crates/ic-wire-canister-evaluation/examples/browser_fixture/mod.rs',process.env.IC_WIRE_BROWSER_WASM]) {
    hashes[path] = createHash('sha256').update(await readFile(path)).digest('hex');
  }
  hashes['PocketIC binary'] = createHash('sha256').update(await readFile(process.env.POCKET_IC_BIN)).digest('hex');
  const sdk = JSON.parse(await readFile('node_modules/@icp-sdk/core/package.json', 'utf8')).version;
  const report = { ...result, browser: browser.version(), node: process.version,
    rustc: execFileSync('rustc', ['-vV'], { encoding: 'utf8' }),
    pocketIc: execFileSync(process.env.POCKET_IC_BIN, ['--version'], { encoding: 'utf8' }).trim(),
    sdk, generator: 'postcard-bindgen 0.9.1', bundleBytes: bundle.length, hashes };
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

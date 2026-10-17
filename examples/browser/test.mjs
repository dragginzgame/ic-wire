import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { withFixtureSession } from '@dragginzgame/ic-testkit-browser';
import { resolve } from 'node:path';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { chromium } from 'playwright';

if (!process.env.POCKET_IC_BIN || !process.env.IC_WIRE_BROWSER_WASM) throw new Error('Run make evaluate-browser with a prepared PocketIC server');
await mkdir('artifacts', { recursive: true });
await withFixtureSession({
  command: resolve('../../target/debug/examples/browser-fixture'), args: [process.cwd()],
  env: { ...process.env, IC_WIRE_BROWSER_CALLER: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7)).getPrincipal().toText() },
  readyPath: 'artifacts/connection.json', logPath: 'artifacts/fixture.log',
  startupMs: 60_000, stopMs: 5_000, pollMs: 50,
}, async connection => {
  let server;
  let browser;
  try {
    const vectors = JSON.parse(await readFile('generated/vectors.json', 'utf8'));
    const nativeVectors = JSON.parse(await readFile('generated/native-vectors.json','utf8'));
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
    const result = await page.evaluate(async ({ connection, vectors, nativeVectors }) => {
      const { run } = await import('/main.js');
      return run(connection, vectors, nativeVectors);
    }, { connection, vectors, nativeVectors });
    await page.screenshot({ path: 'artifacts/browser.png' });
    const sdk = JSON.parse(await readFile('node_modules/@icp-sdk/core/package.json', 'utf8')).version;
    const report = { ...result, vectors, nativeVectors, browser: browser.version(), node: process.version,
      sdk, generator: 'ic-wire Wire::SCHEMA', canisterFeatures: ['browser-fixture'],
      target: 'wasm32-unknown-unknown', profile: 'canister-evaluation', bundleBytes: bundle.length };
    await writeFile('artifacts/report.pending.json', JSON.stringify(report, null, 2) + '\n');
    console.log(JSON.stringify(report, null, 2));
  } finally {
    await browser?.close();
    if (server) await new Promise(resolve => server.close(resolve));
  }
});

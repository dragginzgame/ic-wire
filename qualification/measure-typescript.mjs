// Run from the repository root with the pinned compiler and a populated Cargo cache.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { performance } from 'node:perf_hooks';

const root = 'crates/ic-wire/src';
const lines = { core: 0, typescript: 0 };
for (const path of readdirSync(root, { recursive: true, encoding: 'utf8' })) {
  if (!/\.(rs|ts)$/.test(path) || /(^|\/)tests\.rs$/.test(path)) continue;
  const source = readFileSync(join(root, path), 'utf8');
  lines[path.startsWith('typescript/') ? 'typescript' : 'core'] += source.trimEnd().split('\n').length;
}
const milliseconds = {};
for (const [name, flags] of [['core', []], ['typescript', ['--features', 'typescript']]]) {
  const target = mkdtempSync(join(tmpdir(), 'wire-feature-check-'));
  try {
    const start = performance.now();
    execFileSync('cargo', ['check', '--locked', '-p', 'ic-wire', '--lib',
      '--no-default-features', '--target-dir', target, ...flags], { stdio: 'inherit' });
    milliseconds[name] = Math.round(performance.now() - start);
  } finally {
    rmSync(target, { recursive: true, force: true });
  }
}
console.log(JSON.stringify({ compiler: execFileSync('rustc', ['-Vv'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim(), lines, milliseconds }, null, 2));

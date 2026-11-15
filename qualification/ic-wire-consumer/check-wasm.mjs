// Execute the standalone retained workload; no IC host or imports are provided.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
assert(process.argv.length === 3, 'provide one current consumer artifact');
for (const path of process.argv.slice(2)) {
  const module = new WebAssembly.Module(readFileSync(path));
  assert.deepEqual(WebAssembly.Module.imports(module), [], 'consumer must be SDK/host independent');
  const instance = new WebAssembly.Instance(module);
  assert.equal(instance.exports.main(0, 0), 0, 'consumer roundtrip assertions must pass');
  console.log(`${path}: standalone Wasm workload passed`);
}

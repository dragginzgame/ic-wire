import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { createPayloadCodec, WireError } = await import(pathToFileURL(process.argv[1]));
const codec = createPayloadCodec({ schema: { k: 'bool' }, bytes: 4, nodes: 1, depth: 1 });
for (const [bytes, expected] of JSON.parse(readFileSync(0, 'utf8'))) {
  const decode = () => codec.decode(Uint8Array.from(bytes));
  if (typeof expected === 'boolean') assert.equal(decode(), expected);
  else assert.throws(decode, error => error instanceof WireError && error.code === expected, JSON.stringify(bytes));
}
console.log(true);

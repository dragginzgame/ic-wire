import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
const runtime = await import(pathToFileURL(process.argv[1]));
let checked = 0;
function check(codec, input, expected) {
  if (expected === null) assert.throws(() => codec.decode(input), error => error instanceof runtime.WireError && error.code === 'malformed');
  else assert.deepEqual(codec.decode(input), expected);
  checked++;
}
for (const k of ['u', 'i']) {
  const codec = runtime.createPayloadCodec({ schema: { k, bits: 16 }, bytes: 32, nodes: 8, depth: 4 });
  const value = n => k === 'u' ? n : (n >>> 1) ^ -(n & 1);
  check(codec, new Uint8Array(), null);
  for (let first = 0; first < 256; first++) {
    check(codec, Uint8Array.of(first), first < 128 ? value(first) : null);
    for (let second = 0; second < 256; second++) {
      check(codec, Uint8Array.of(first, second), first >= 128 && second < 128 ? value((first & 127) | second << 7) : null);
    }
  }
}
const boolean = runtime.createPayloadCodec({ schema: { k: 'bool' }, bytes: 32, nodes: 8, depth: 4 });
for (let byte = 0; byte < 256; byte++) check(boolean, Uint8Array.of(byte), byte < 2 ? byte === 1 : null);
process.stdout.write(JSON.stringify({ checked }));

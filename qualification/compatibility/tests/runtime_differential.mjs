import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
const url = pathToFileURL(process.argv[1]);
const candidate = await import(url);
const reference = await import(new URL('./oracle-runtime.ts', url));
function outcome(runtime, action) {
  try { return { value: action() }; }
  catch (error) { assert(error instanceof runtime.WireError); return { error: error.code }; }
}
let checked = 0;
function compare(schema, inputs, values = []) {
  const meta = { schema, bytes: 32, nodes: 8, depth: 4 };
  const a = candidate.createPayloadCodec(meta), b = reference.createPayloadCodec(meta);
  for (const input of inputs) {
    assert.deepEqual(outcome(candidate, () => a.decode(input)), outcome(reference, () => b.decode(input)), `${JSON.stringify(schema)} ${input}`);
    checked++;
  }
  for (const value of values) {
    assert.deepEqual(outcome(candidate, () => a.encode(value)), outcome(reference, () => b.encode(value)));
    checked++;
  }
}
function* pairs() {
  yield new Uint8Array();
  for (let first = 0; first < 256; first++) {
    yield Uint8Array.of(first);
    for (let second = 0; second < 256; second++) yield Uint8Array.of(first, second);
  }
}
for (const k of ['u', 'i']) compare({ k, bits: 16 }, pairs(), [-32769, -32768, -1, 0, 32767, 32768, 65535, 65536, NaN, Infinity, 1.5, 1n]);
for (const schema of [{ k: 'bool' }, { k: 'u', bits: 8 }, { k: 'i', bits: 8 }]) {
  compare(schema, Array.from({ length: 256 }, (_, n) => Uint8Array.of(n)));
}
for (const bits of [32, 64, 128]) {
  const bytes = Math.ceil(bits / 7);
  const inputs = [new Uint8Array(), Uint8Array.of(128, 0)];
  for (let length = 1; length <= bytes + 1; length++) {
    inputs.push(new Uint8Array(length).fill(128), new Uint8Array(length).fill(255));
  }
  for (let end = 0; end < 256; end++) {
    const input = new Uint8Array(bytes).fill(255); input[bytes - 1] = end; inputs.push(input);
  }
  for (const k of ['u', 'i']) compare({ k, bits }, inputs);
}
compare({ k: 'text' }, [Uint8Array.of(1, 255), Uint8Array.of(2, 192, 128)], ['\ud800', '\udc00', '\ud800x', '😀', '\ufeff']);
process.stdout.write(JSON.stringify({ checked }));

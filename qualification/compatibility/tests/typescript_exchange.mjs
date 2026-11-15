// Qualification consumer of candidate metadata and the pinned published runtime.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { createPayloadCodec, createCodec, WireError } = await import(pathToFileURL(process.argv[1]).href);
const { types, cases, policy } = JSON.parse(readFileSync(0, 'utf8'));
const schema = { k: 'ref', name: 'Record' };
const payload = createPayloadCodec({ schema, ...policy }, types);
const frame = createCodec({ id: '7', request: schema, response: schema,
  requestBytes: policy.bytes, responseBytes: policy.bytes,
  requestNodes: policy.nodes, responseNodes: policy.nodes,
  requestDepth: policy.depth, responseDepth: policy.depth }, types);
const malformed = error => error instanceof WireError && error.code === 'malformed';
const outputs = [];
for (const [caseIndex, item] of cases.entries()) {
  const value = Object.assign(Object.create(null), item.value, {
    bytes: new Uint8Array(item.value.bytes), packed: new Int16Array(item.value.packed),
  });
  const encoded = [];
  for (const [bytes, encode, decode, header] of [
    [item.payload, payload.encode, payload.decode, 0],
    [item.request, frame.encodeRequest, frame.decodeRequest, 11],
    [item.response, frame.encodeResponse, frame.decodeResponse, 11],
  ]) {
    const backing = new Uint8Array(bytes.length + 2);
    backing[0] = 99; backing.set(bytes, 1); backing[backing.length - 1] = 88;
    const selected = backing.subarray(1, backing.length - 1);
    const decoded = decode(selected);
    assert.deepEqual(decoded, value);
    backing.fill(0);
    assert.deepEqual(decoded, value); // Decoded bulk fields own their storage.
    const result = encode(value);
    assert.deepEqual(Array.from(result), bytes);
    encoded.push(Array.from(result));
    for (let end = 0; end < bytes.length; end++) {
      // Reference bulk length admission checks the available input span first.
      // These truncated prefixes report limit; other truncations are malformed.
      const limitEnds = [[], [5], [5], [10, 11, 12]][caseIndex];
      const code = limitEnds.includes(end - header) ? 'limit' : 'malformed';
      assert.throws(() => decode(new Uint8Array(bytes.slice(0, end))), error => error instanceof WireError && error.code === code);
    }
    assert.throws(() => decode(new Uint8Array([...bytes, 0])), malformed);
  }
  outputs.push(encoded);
}
const sample = { ...cases.at(-1).value, bytes: new Uint8Array([170,187]), packed: new Int16Array([1,-1]) };
const retained = payload.encode(sample);
sample.bytes.fill(0); sample.packed.fill(0);
assert.deepEqual(Array.from(retained), cases.at(-1).payload);
assert.throws(() => payload.encode({ ...sample, number: 256 }), error => error instanceof WireError && error.code === 'limit');
assert.throws(() => payload.encode({ ...sample, bytes: new Uint8Array(4) }), error => error instanceof WireError && error.code === 'limit');
const exact = createPayloadCodec({ schema, ...policy, bytes: retained.length }, types);
assert.deepEqual(Array.from(exact.encode(payload.decode(retained))), Array.from(retained));
const short = createPayloadCodec({ schema, ...policy, bytes: retained.length - 1 }, types);
assert.throws(() => short.decode(retained), error => error instanceof WireError && error.code === 'limit');
assert.throws(() => short.encode(payload.decode(retained)), error => error instanceof WireError && error.code === 'limit');
process.stdout.write(JSON.stringify(outputs));

// Qualification consumer of candidate metadata and the pinned published runtime.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { createPayloadCodec, createCodec, WireError } = await import(pathToFileURL(process.argv[1]).href);
const { types, cases, policy, chain, shared } = JSON.parse(readFileSync(0, 'utf8'));
const schema = { k: 'ref', name: 'Record' };
const client = await import(new URL('./client.ts', pathToFileURL(process.argv[1])).href);
const adapter = await import(new URL('./runtime-adapter.ts', pathToFileURL(process.argv[1])).href);
assert.deepEqual(Object.keys(client), ['recordCodec']);
assert.deepEqual(adapter.observed, { schema, ...policy });
assert(Object.isFrozen(client.recordCodec));
await assert.rejects(import(new URL('./bad-client.ts', pathToFileURL(process.argv[1])).href), { message: 'IC Wire runtime format mismatch' });
const payload = client.recordCodec;
const frame = createCodec({ id: '7', request: schema, response: schema,
  requestBytes: policy.bytes, responseBytes: policy.bytes,
  requestNodes: policy.nodes, responseNodes: policy.nodes,
  requestDepth: policy.depth, responseDepth: policy.depth }, types);
const malformed = error => error instanceof WireError && error.code === 'malformed';
const limited = error => error instanceof WireError && error.code === 'limit';
const outputs = [];
for (const [caseIndex, item] of cases.entries()) {
  const value = Object.assign(Object.create(null), item.value, {
    bytes: new Uint8Array(item.value.bytes), packed: new Int16Array(item.value.packed),
  });
  const encoded = [];
  // One record, six fields, an enum payload, optional Some and list elements.
  // Reference JS counts values; these are not Rust allocation/node allowances.
  const minimumNodes = [8, 9, 9, 11][caseIndex];
  for (const resource of ['bytes', 'nodes', 'depth']) {
    const minimum = { bytes: item.payload.length, nodes: minimumNodes, depth: 3 }[resource];
    for (const delta of [-1, 0, 1]) {
      const codec = createPayloadCodec({ schema, ...policy, [resource]: minimum + delta }, types);
      if (delta < 0) {
        assert.throws(() => codec.encode(value), limited);
        assert.throws(() => codec.decode(new Uint8Array(item.payload)), limited);
      } else {
        assert.deepEqual(Array.from(codec.encode(value)), item.payload);
        assert.deepEqual(codec.decode(new Uint8Array(item.payload)), value);
      }
    }
  }
  // Vary one direction/resource while the opposite direction stays sufficient.
  for (const direction of ['request', 'response']) {
    for (const resource of ['Bytes', 'Nodes', 'Depth']) {
      const minimum = { Bytes: item.request.length, Nodes: minimumNodes, Depth: 3 }[resource];
      for (const delta of [-1, 0, 1]) {
        const codec = createCodec({ id: '7', request: schema, response: schema,
          requestBytes: policy.bytes, responseBytes: policy.bytes,
          requestNodes: policy.nodes, responseNodes: policy.nodes,
          requestDepth: policy.depth, responseDepth: policy.depth,
          [direction + resource]: minimum + delta }, types);
        const encode = direction === 'request' ? codec.encodeRequest : codec.encodeResponse;
        const decode = direction === 'request' ? codec.decodeRequest : codec.decodeResponse;
        const bytes = item[direction];
        if (delta < 0) {
          assert.throws(() => encode(value), limited);
          assert.throws(() => decode(new Uint8Array(bytes)), limited);
        } else {
          assert.deepEqual(Array.from(encode(value)), bytes);
          assert.deepEqual(decode(new Uint8Array(bytes)), value);
        }
        const otherEncode = direction === 'request' ? codec.encodeResponse : codec.encodeRequest;
        const otherDecode = direction === 'request' ? codec.decodeResponse : codec.decodeRequest;
        const otherBytes = item[direction === 'request' ? 'response' : 'request'];
        assert.deepEqual(Array.from(otherEncode(value)), otherBytes);
        assert.deepEqual(otherDecode(new Uint8Array(otherBytes)), value);
      }
    }
  }
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
const record = fields => Object.assign(Object.create(null), fields);
const chainValue = record({ value: 1, next: { some: record({ value: 128, next: null }) } });
const leaf = record({ value: 7, next: null });
// Same JS object in two fields: schemas share structure, bytes contain two values.
const sharedValue = record({ left: leaf, right: leaf });
const recursiveOutputs = [];
for (const [name, value, literal, nodes, depth] of [
  ['Chain', chainValue, [1, 1, 128, 1, 0], 6, 4],
  ['Shared', sharedValue, [7, 0, 7, 0], 7, 3],
]) {
  assert.deepEqual(name === 'Chain' ? chain : shared, literal);
  const schema = { k: 'ref', name };
  const codec = createPayloadCodec({ schema, ...policy, nodes, depth }, types);
  const bytes = codec.encode(value);
  assert.deepEqual(Array.from(bytes), literal);
  const decoded = codec.decode(new Uint8Array(literal));
  assert.deepEqual(decoded, value);
  if (name === 'Shared') {
    assert.notEqual(decoded.left, decoded.right);
    decoded.left.value = 9;
    assert.equal(decoded.right.value, 7);
  }
  for (const allowance of [{ nodes: nodes - 1, depth }, { nodes, depth: depth - 1 }]) {
    const short = createPayloadCodec({ schema, ...policy, ...allowance }, types);
    assert.throws(() => short.encode(value), limited);
    assert.throws(() => short.decode(new Uint8Array(literal)), limited);
  }
  recursiveOutputs.push(Array.from(bytes));
}
const cyclic = record({ value: 1, next: null });
cyclic.next = { some: cyclic };
const bounded = createPayloadCodec({ schema: { k: 'ref', name: 'Chain' }, ...policy, depth: 4 }, types);
assert.throws(() => bounded.encode(cyclic), limited);
process.stdout.write(JSON.stringify({ records: outputs, chain: recursiveOutputs[0], shared: recursiveOutputs[1] }));

const { createPayloadCodec } = await import('./compiled/wire-runtime.js');
const plainPayload = value => Array.isArray(value) ? value.map(plainPayload)
  : value !== null && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k,v]) => [k,plainPayload(v)])) : value;
function checkPayload(codec, value, expected) {
  const bytes = Uint8Array.from(expected);
  assert.deepEqual(codec.encode(value), bytes);
  assert.deepEqual(plainPayload(codec.decode(bytes)), plainPayload(value));
  assert.deepEqual(codec.encode(codec.decode(bytes)), bytes);
  for (let end = 0; end < bytes.length; end++) assert.throws(() => codec.decode(bytes.slice(0, end)));
  rejects(() => codec.decode(Uint8Array.from([...bytes,0])), 'malformed');
}
const payloadProbe = (schema, limits = {}, types = {}, checks = {}) => createPayloadCodec({schema,bytes:64,nodes:64,depth:16,...limits}, types, checks);
const zero = payloadProbe({k:'unit'}, {bytes:0,nodes:1,depth:1});
assert.deepEqual(zero.encode(null), new Uint8Array());
assert.equal(zero.decode(new Uint8Array()), null);
rejects(() => zero.decode(new Uint8Array([0])), 'limit');
for (const invalid of [{bytes:-1},{bytes:0.5},{nodes:0},{depth:0},{bytes:Number.MAX_SAFE_INTEGER+1}]) {
  rejects(() => payloadProbe({k:'unit'}, invalid), 'value');
}
for (const [schema, malformed] of [[{k:'u',bits:16},[128,0]],[{k:'u',bits:16},[255,255,4]],[{k:'bool'},[2]],[{k:'text'},[1,255]],[{k:'option',item:{k:'unit'}},[2]]]) {
  rejects(() => payloadProbe(schema).decode(Uint8Array.from(malformed)), 'malformed');
}
for (const limit of [{bytes:1},{nodes:1},{depth:1}]) {
  const codec = payloadProbe({k:'tuple',items:[{k:'u',bits:16}]}, limit);
  rejects(() => codec.encode([128]), 'limit');
  rejects(() => codec.decode(new Uint8Array([128,1])), 'limit');
}
const owned = payloadProbe({k:'bytes'}), input = Buffer.from([2,1,2]);
owned.decode(input).fill(0);
assert.deepEqual([...input], [2,1,2]);
const checked = payloadProbe({k:'u',bits:64,check:'positive'}, {}, {}, {positive:v => { if (v <= 0n) throw new WireError('value'); }});
rejects(() => checked.encode(0n), 'value');
rejects(() => checked.decode(new Uint8Array([0])), 'value');
// Consumer-owned presentation facade: upstream codecs retain their wire values.
class Counter { constructor(value) { this.value = value; } }
const counter = {encode:v => checked.encode(v.value), decode:b => new Counter(checked.decode(b))};
assert.equal(counter.decode(counter.encode(new Counter(7n))).value, 7n);
assert.deepEqual(counter.encode(new Counter(7n)), new Uint8Array([7]));
const {counter:typedCounter, Counter:TypedCounter} = await import('./compiled/payload-facade.js');
assert.equal(typedCounter.decode(typedCounter.encode(new TypedCounter(7n))).value, 7n);
assert.throws(() => new TypedCounter(-1n), RangeError);

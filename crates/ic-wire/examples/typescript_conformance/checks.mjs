import assert from 'node:assert/strict';
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';
import { WireError } from './compiled/wire-runtime.js';
import { nativeTree, prototypeName } from './compiled/native.js';
const normalize = value => value instanceof Uint8Array || value instanceof Int16Array ? value : Array.isArray(value) ? value.map(normalize) : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k,v]) => [k, normalize(v)])) : value;
function check(codec, value, request, response) {
  for (const [encode, decode, bytes] of [[codec.encodeRequest, codec.decodeRequest, request], [codec.encodeResponse, codec.decodeResponse, response]]) {
    const input = Uint8Array.from(bytes);
    assert.deepEqual(encode(value), input);
    assert.deepEqual(normalize(decode(input)), normalize(value));
    for (let end = 0; end < bytes.length; end++) assert.throws(() => decode(input.slice(0,end)), WireError);
    assert.throws(() => decode(Uint8Array.from([...bytes,0])), WireError);
    for (const offset of [0,2,10]) { const bad = input.slice(); bad[offset] ^= 1; assert.throws(() => decode(bad), WireError); }
    // A subarray must respect its byteOffset; decoded byte values own their storage.
    const padded = new Uint8Array(input.length+4); padded.set(input,2);
    const decoded = decode(padded.subarray(2,-2)); padded.fill(0);
    assert.deepEqual(normalize(decoded), normalize(value));
    // Node Buffer.slice() aliases storage; decoded bytes must still be owned.
    const nodeInput = Buffer.alloc(input.length+4); nodeInput.set(input,2);
    const nodeDecoded = decode(nodeInput.subarray(2,-2)); nodeInput.fill(0);
    assert.deepEqual(normalize(nodeDecoded), normalize(value));
  }
}
// The descriptor here exercises hostile inputs without duplicating Rust schemas.
import { createCodec } from './compiled/wire-runtime.js';
function probe(schema, nodes = 32, depth = 8) {
  return createCodec({id:'1',request:schema,response:schema,requestBytes:128,responseBytes:128,requestNodes:nodes,responseNodes:nodes,requestDepth:depth,responseDepth:depth});
}
const header = [73,87,1,0,0,0,0,0,0,0,0];
const frame = (...payload) => Uint8Array.from([...header,...payload]);
// Mutating an owned decode result must not change the original Buffer frame.
for (const direction of ['Request','Response']) {
  const codec = probe({k:'bytes',max:8});
  const bytes = Buffer.from(codec['encode'+direction](new Uint8Array([1,2,3])));
  const original = Uint8Array.from(bytes);
  codec['decode'+direction](bytes).fill(0);
  assert.deepEqual(Uint8Array.from(bytes),original);
}
assert.deepEqual(prototypeName.encodeRequest(null), frame());
assert.equal(prototypeName.decodeRequest(frame()), null);
const rejects = (action, code) => assert.throws(action, error => error instanceof WireError && error.code === code);
for (const payload of [[128,0],[255,255,4],[128,128,128]]) rejects(() => probe({k:'u',bits:16}).decodeRequest(frame(...payload)), 'malformed');
rejects(() => probe({k:'bool'}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'option',item:{k:'unit'}}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'result',ok:{k:'unit'},err:{k:'unit'}}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'enum',variants:[{name:'Unit',id:128,value:{k:'unit'}}]}).decodeRequest(frame(3)), 'malformed');
for (const payload of [[1,255],[2,192,128],[3,237,160,128],[1,239]]) rejects(() => probe({k:'text',max:8}).decodeRequest(frame(...payload)), 'malformed');
for (const k of ['bytes','text','list']) rejects(() => probe({k,max:4,item:{k:'unit'}}).decodeRequest(frame(5)), 'limit');
// Length fast paths must preserve bounds and the full u64 prefix error contract.
for (const k of ['bytes','text','list']) {
  for (let length = 0; length < 128; length++) {
    const schema = {k,max:length,item:{k:'unit'}};
    const codec = createCodec({id:'1',request:schema,response:schema,requestBytes:256,responseBytes:256,requestNodes:256,responseNodes:256,requestDepth:8,responseDepth:8});
    const payload = k === 'list' ? [] : Array(length).fill(97);
    for (const direction of ['Request','Response']) {
      const bytes = frame(length,...payload); bytes[10] = direction === 'Request' ? 0 : 1;
      assert.equal(codec['decode'+direction](bytes).length,length);
      const excessive = frame(...(length === 127 ? [128,1] : [length+1])); excessive[10] = bytes[10];
      rejects(() => codec['decode'+direction](excessive),'limit');
    }
  }
  const codec = probe({k,max:127,item:{k:'unit'}});
  for (const payload of [[],[128],[128,0],[255,255,255,255,255,255,255,255,255,2],Array(10).fill(128)]) {
    rejects(() => codec.decodeRequest(frame(...payload)),'malformed');
  }
  for (const payload of [[128,1],[255,255,255,255,255,255,255,255,255,1]]) {
    rejects(() => codec.decodeRequest(frame(...payload)),'limit');
  }
}
rejects(() => probe({k:'list',max:4,item:{k:'unit'}},3).decodeRequest(frame(4)), 'limit');
rejects(() => probe({k:'option',item:{k:'unit'}},32,1).decodeRequest(frame(1)), 'limit');
for (const bits of [8,16,32]) for (const value of [-1,2**bits,0.5,NaN,Infinity]) rejects(() => probe({k:'u',bits}).encodeRequest(value), Number.isSafeInteger(value) ? 'limit' : 'value');
rejects(() => probe({k:'u',bits:64}).encodeRequest(9007199254740992), 'value');
rejects(() => probe({k:'i',bits:128}).encodeRequest(-(1n<<127n)-1n), 'limit');
rejects(() => probe({k:'text',max:8}).encodeRequest('\ud800'), 'value');
rejects(() => probe({k:'text',max:3}).encodeRequest('🦀'), 'limit');
rejects(() => probe({k:'bytes',max:4}).encodeRequest(new Uint8Array(5)), 'limit');
rejects(() => probe({k:'array',max:2,item:{k:'unit'}}).encodeRequest([null]), 'limit');
for (const value of [{}, {some:null,extra:1}]) rejects(() => probe({k:'option',item:{k:'unit'}}).encodeRequest(value), 'value');
rejects(() => probe({k:'result',ok:{k:'unit'},err:{k:'unit'}}).encodeRequest({ok:null,err:null}), 'value');
rejects(() => probe({k:'record',fields:[]}).encodeRequest({extra:1}), 'value');
rejects(() => probe({k:'enum',variants:[]}).encodeRequest({tag:'Unknown',value:null}), 'value');
// Enum IDs are unsigned 32-bit tags, independent of declaration order.
for (const id of [0,127,128,16383,16384,2097151,2097152,268435455,268435456,4294967295]) {
  const codec = probe({k:'enum',variants:[{name:'Chosen',id,value:{k:'unit'}},{name:'Other',id:3,value:{k:'unit'}}]});
  const value = {tag:'Chosen',value:null}, payload = [];
  let remaining = BigInt(id);
  do { const byte = Number(remaining & 127n); remaining >>= 7n; payload.push(remaining ? byte|128 : byte); } while (remaining);
  for (const direction of ['Request','Response']) {
    const bytes = frame(...payload); bytes[10] = direction === 'Request' ? 0 : 1;
    assert.deepEqual(codec['encode'+direction](value),bytes);
    assert.deepEqual(codec['decode'+direction](bytes),value);
    for (const bad of [[2],[128],[128,0],[255,255,255,255,16],[128,128,128,128,128]]) {
      const malformed = frame(...bad); malformed[10] = bytes[10];
      rejects(() => codec['decode'+direction](malformed),'malformed');
    }
  }
}
// Numeric integer metadata bounds apply exactly, including to bigint payloads.
for (const kind of ['u','i']) for (const bits of [8,16,32,64,128]) {
  const raw = probe({k:kind,bits});
  const bounded = probe({k:kind,bits,min:1,max:2});
  const value = n => bits > 32 ? BigInt(n) : n;
  for (const n of [1,2]) {
    const bytes = bounded.encodeRequest(value(n));
    assert.deepEqual(bytes,raw.encodeRequest(value(n)));
    assert.equal(bounded.decodeRequest(bytes),value(n));
  }
  for (const n of [0,3]) {
    rejects(() => bounded.encodeRequest(value(n)),'value');
    rejects(() => bounded.decodeRequest(raw.encodeRequest(value(n))),'malformed');
    rejects(() => bounded.encodeResponse(value(n)),'value');
    rejects(() => bounded.decodeResponse(raw.encodeResponse(value(n))),'malformed');
  }
}
// Independently construct canonical small-integer bytes through a bigint oracle.
for (const bits of [8,16,32]) for (const kind of ['u','i']) {
  const signed = kind === 'i', low = signed ? -(2 ** (bits-1)) : 0, high = 2 ** (bits-(signed?1:0))-1;
  const values = new Set([low,high,0,1]);
  for (const shift of [7,14,21,28]) for (const n of [2**shift-1,2**shift,2**shift+1]) {
    values.add(n); values.add(-n); values.add(Math.floor(n/2)); values.add(-Math.floor(n/2));
  }
  const codec = probe({k:kind,bits});
  for (const value of values) {
    if (value < low || value > high) continue;
    let n = BigInt(value);
    const payload = [];
    if (bits === 8) payload.push(Number(n & 255n));
    else {
      if (signed) n = n < 0n ? -2n*n-1n : 2n*n;
      do { const byte = Number(n & 127n); n >>= 7n; payload.push(n ? byte|128 : byte); } while(n);
    }
    for (const direction of ['Request','Response']) {
      const expected = frame(...payload); expected[10] = direction === 'Response' ? 1 : 0;
      assert.deepEqual(codec['encode'+direction](value),expected);
      assert.equal(codec['decode'+direction](expected),value === 0 ? 0 : value);
      for (let end=11;end<expected.length;end++) rejects(()=>codec['decode'+direction](expected.slice(0,end)),'malformed');
      if (bits > 8) {
        const overlong = new Uint8Array(expected.length+1); overlong.set(expected); overlong[expected.length-1] |= 128;
        rejects(()=>codec['decode'+direction](overlong),'malformed');
      }
    }
  }
  for(const value of [low-1,high+1]) rejects(()=>codec.encodeRequest(value),'limit');
}
// Collection limits count every nested value, including zero-byte units.
{
  const schema = {k:'list',max:2,item:{k:'list',max:3,item:{k:'unit'}}};
  const value = [[null,null,null],[null,null,null]], exact = probe(schema,9,3);
  for(const direction of ['Request','Response']) {
    const bytes = exact['encode'+direction](value);
    assert.deepEqual(exact['decode'+direction](bytes),value);
    for(const codec of [probe(schema,8,3),probe(schema,9,2)]) {
      rejects(()=>codec['encode'+direction](value),'limit');
      rejects(()=>codec['decode'+direction](bytes),'limit');
    }
    const oversized = bytes.slice(); oversized[12] = 4;
    rejects(()=>exact['decode'+direction](oversized),'limit');
  }
}
// Text prefix boundaries, tight budgets and independent compact returned frames.
for(const text of ['', 'a'.repeat(127), 'a'.repeat(128), 'é'.repeat(64), '🦀'.repeat(32), '\ufefftext']) {
  const schema = {k:'text',max:512};
  const meta = {id:'1',request:schema,response:schema,requestBytes:1024,responseBytes:1024,requestNodes:1,responseNodes:1,requestDepth:1,responseDepth:1};
  const codec = createCodec(meta), payload = new TextEncoder().encode(text);
  let length = BigInt(payload.length); const prefix = [];
  do {const b=Number(length&127n);length>>=7n;prefix.push(length?b|128:b);}while(length);
  for(const direction of ['Request','Response']) {
    const expected = frame(...prefix,...payload);expected[10]=direction==='Response'?1:0;
    const exact = createCodec({...meta,requestBytes:expected.length,responseBytes:expected.length});
    const bytes = exact['encode'+direction](text);
    assert.deepEqual(bytes,expected);assert.equal(bytes.buffer.byteLength,bytes.length);
    assert.equal(codec['decode'+direction](bytes),text);
    codec['encode'+direction]('other');assert.deepEqual(bytes,expected);
    const short=createCodec({...meta,requestBytes:expected.length-1,responseBytes:expected.length-1});
    rejects(()=>short['encode'+direction](text),'limit');
  }
}
for(const text of ['\ud800','\udc00','\ud800\ud800','\udc00\ud800','a\ud800','\udc00🦀']) {
  rejects(()=>probe({k:'text',max:64}).encodeRequest(text),'value');
  rejects(()=>probe({k:'text',max:64}).encodeResponse(text),'value');
  rejects(()=>probe({k:'text',max:0}).encodeRequest(text),'limit');
}
console.log('Hostile-input and outgoing-value rejection checks passed');

for (const bits of [32,64]) {
  const codec = probe({k:'float',bits});
  for (const value of [NaN,Infinity,-Infinity,'1',null]) rejects(() => codec.encodeRequest(value), 'value');
  const patterns = bits === 32 ? [0x7f800000n,0xff800000n,0x7fc00000n,0x7f800001n,0xffc00123n] : [0x7ff0000000000000n,0xfff0000000000000n,0x7ff8000000000000n,0x7ff0000000000001n,0xfff8000000000123n];
  for (const pattern of patterns) {
    const payload = new Uint8Array(bits/8), view = new DataView(payload.buffer);
    if (bits === 32) view.setUint32(0,Number(pattern),true); else view.setBigUint64(0,pattern,true);
    rejects(() => codec.decodeRequest(frame(...payload)), 'malformed');
  }
}
for (const value of [0.1, Number.MAX_VALUE, Number.MIN_VALUE]) rejects(() => probe({k:'float',bits:32}).encodeRequest(value), 'value');
// Explicit float bounds apply in both directions, including their endpoints.
for (const bits of [32,64]) {
  const raw = probe({k:'float',bits});
  const bounded = probe({k:'float',bits,min:0,max:1});
  for (const value of [-0,0,0.5,1]) {
    const bytes = bounded.encodeRequest(value);
    assert.deepEqual(bytes,raw.encodeRequest(value));
    assert.ok(Object.is(bounded.decodeRequest(bytes),value));
  }
  for (const value of [-1,2]) {
    rejects(() => bounded.encodeRequest(value),'value');
    rejects(() => bounded.decodeRequest(raw.encodeRequest(value)),'malformed');
    rejects(() => bounded.encodeResponse(value),'value');
    rejects(() => bounded.decodeResponse(raw.encodeResponse(value)),'malformed');
  }
  const schema = {k:'float',bits};
  const exact = {id:'1',request:schema,response:schema,requestBytes:11+bits/8,responseBytes:11+bits/8,requestNodes:1,responseNodes:1,requestDepth:1,responseDepth:1};
  assert.equal(createCodec(exact).encodeRequest(0.5).length,11+bits/8);
  rejects(() => createCodec({...exact,requestBytes:10+bits/8}).encodeRequest(0.5),'limit');
}
// Encoding float arrays needs only the writer's explicit buffer/view allocation.
// The final slice still allocates independent returned storage.
{
  const ArrayType = globalThis.Uint8Array, ViewType = globalThis.DataView;
  let arrays = 0, views = 0;
  try {
    globalThis.Uint8Array = new Proxy(ArrayType, {construct(target, args) { arrays++; return Reflect.construct(target,args); }});
    globalThis.DataView = new Proxy(ViewType, {construct(target, args) { views++; return Reflect.construct(target,args); }});
    probe({k:'array',max:4,item:{k:'float',bits:64}}).encodeRequest([0,0.5,-0,1]);
    assert.equal(arrays,1);
    assert.equal(views,1);
  } finally {
    globalThis.Uint8Array = ArrayType;
    globalThis.DataView = ViewType;
  }
}
// Float decoding shares the header view, including offset-backed input.
for (const bits of [32,64]) {
  const codec = probe({k:'array',max:4,item:{k:'float',bits}});
  const values = [0,0.5,-0,1];
  const bytes = codec.encodeRequest(values);
  const backing = new Uint8Array(bytes.length+8);
  backing.set(bytes,3);
  const input = backing.subarray(3,3+bytes.length);
  const ViewType = globalThis.DataView, subarray = Uint8Array.prototype.subarray;
  let views = 0, slices = 0;
  try {
    globalThis.DataView = new Proxy(ViewType, {construct(target,args) { views++; return Reflect.construct(target,args); }});
    Uint8Array.prototype.subarray = function(...args) { slices++; return subarray.apply(this,args); };
    assert.deepEqual(codec.decodeRequest(input),values);
    assert.equal(views,1);
    assert.equal(slices,0);
  } finally {
    globalThis.DataView = ViewType;
    Uint8Array.prototype.subarray = subarray;
  }
  for (let end = 11; end < bytes.length; end++) rejects(() => codec.decodeRequest(bytes.slice(0,end)),'malformed');
}
console.log('Finite-float canonical bits and rejection checks passed');

// Native Serde consumers supply concrete recursive metadata and explicit traversal
// allowances. They share the finite codec's primitives and envelope.
const nativeTypes = {
  Node: { k:'enum', variants:[
    {name:'Leaf',id:0,value:{k:'tuple',items:[{k:'text'},{k:'newtype',item:{k:'u',bits:64}}]}},
    {name:'Branch',id:1,value:{k:'list',item:{k:'ref',name:'Node'}}},
  ]},
};
const nativeMeta = {id:'1',request:{k:'ref',name:'Node'},response:{k:'ref',name:'Node'},requestBytes:128,responseBytes:128,requestNodes:32,responseNodes:32,requestDepth:8,responseDepth:8};
const native = createCodec(nativeMeta,nativeTypes);
const tree = {tag:'Branch',value:[{tag:'Leaf',value:['é',128n]}]};
const treeBytes = frame(...nativePayload);
check(nativeTree, tree, [...treeBytes], [...treeBytes.subarray(0, 10), 1, ...nativePayload]);
assert.deepEqual(native.encodeRequest(tree),treeBytes);
assert.deepEqual(normalize(native.decodeRequest(treeBytes)),tree);
assert.deepEqual(native.encodeRequest(native.decodeRequest(treeBytes)),treeBytes);
for (let end=0;end<treeBytes.length;end++) assert.throws(()=>native.decodeRequest(treeBytes.slice(0,end)),WireError);
rejects(()=>createCodec({...nativeMeta,requestNodes:undefined},nativeTypes),'value');
rejects(()=>createCodec({...nativeMeta,requestNodes:2},nativeTypes).decodeRequest(treeBytes),'limit');
rejects(()=>createCodec({...nativeMeta,requestDepth:2},nativeTypes).decodeRequest(treeBytes),'limit');
rejects(()=>native.decodeRequest(frame(1,255,255,255,255,15)),'limit');
rejects(()=>native.encodeRequest({tag:'Leaf',value:['é']}),'value');
for (const types of [{Node:{k:'ref',name:'Node'}},{Node:{k:'ref',name:'Missing'}}]) {
  rejects(()=>createCodec(nativeMeta,types).encodeRequest(tree),'value');
  rejects(()=>createCodec(nativeMeta,types).decodeRequest(treeBytes),'value');
}
const checked = {...nativeMeta,request:{k:'text',check:'consumer'},response:{k:'text',check:'consumer'}};
const checks = {consumer(value) { if(value !== 'ok') throw new WireError('value'); }};
const scalar = createCodec(checked,{},checks);
assert.equal(scalar.decodeRequest(scalar.encodeRequest('ok')),'ok');
rejects(()=>scalar.encodeRequest('bad'),'value');
rejects(()=>scalar.decodeRequest(frame(3,98,97,100)),'value');
rejects(()=>createCodec(checked).encodeRequest('ok'),'value');
// Reference-level checks must survive every alias, including missing validators.
for (const location of ['request', 'alias', 'concrete']) {
  const request = {k:'ref',name:'Alias'};
  const types = {Alias:{k:'ref',name:'Text'},Text:{k:'text'}};
  (location === 'request' ? request : location === 'alias' ? types.Alias : types.Text).check = 'consumer';
  const codec = createCodec({...nativeMeta,request},types,checks);
  assert.equal(codec.decodeRequest(codec.encodeRequest('ok')),'ok');
  rejects(()=>codec.encodeRequest('bad'),'value');
  rejects(()=>codec.decodeRequest(frame(3,98,97,100)),'value');
  rejects(()=>createCodec({...nativeMeta,request},types).encodeRequest('ok'),'value');
  rejects(()=>createCodec({...nativeMeta,request},types).decodeRequest(frame(2,111,107)),'value');
}
// Registry size is computed once per operation, not once per referenced value.
// Changes between operations remain visible; cyclic aliases still fail.
{
  const types = Object.fromEntries(Array.from({length: 1000}, (_, i) => [`T${i}`, {k:'u',bits:8}]));
  const schema = {k:'list',item:{k:'ref',name:'T0'}};
  const codec = createCodec({...nativeMeta,request:schema,response:schema,requestNodes:128,responseNodes:128},types);
  const originalKeys = Object.keys;
  let scans = 0;
  try {
    Object.keys = value => { if (value === types) scans++; return originalKeys(value); };
    const values = Array(100).fill(7);
    const bytes = codec.encodeRequest(values);
    assert.equal(scans,1);
    assert.deepEqual(codec.decodeRequest(bytes),values);
    assert.equal(scans,2);
    types.T0 = {k:'ref',name:'Added'};
    types.Added = {k:'u',bits:8};
    assert.deepEqual(codec.encodeRequest(values),bytes);
    assert.deepEqual(codec.decodeRequest(bytes),values);
    types.Added = {k:'ref',name:'T0'};
    rejects(() => codec.encodeRequest(values),'value');
    rejects(() => codec.decodeRequest(bytes),'value');
  } finally {
    Object.keys = originalKeys;
  }
}
// Short strings must not allocate a frame-sized UTF-8 scratch buffer per item.
const encodeInto = TextEncoder.prototype.encodeInto;
try {
  TextEncoder.prototype.encodeInto = function(text, buffer) {
    assert.ok(buffer.length <= text.length * 3);
    return encodeInto.call(this,text,buffer);
  };
  const codec = probe({k:'list',item:{k:'text'}});
  const strings = ['', 'a', 'é', '🦀', '\ufeff'];
  assert.deepEqual(codec.decodeRequest(codec.encodeRequest(strings)),strings);
} finally {
  TextEncoder.prototype.encodeInto = encodeInto;
}
console.log('Recursive native metadata, tuples, aliases, scalar validation and explicit traversal refusal passed');

for (const schema of [{k:'list',item:{k:'unit'}}, {k:'array',max:1,item:{k:'unit'}}, {k:'tuple',items:[{k:'unit'}]}]) {
  rejects(() => createCodec({...nativeMeta,request:schema},{}).encodeRequest(new Array(1)), 'value');
}
function mutation(codec, direction, bytes, accepted) {
  const input = Uint8Array.from(bytes);
  const decode = direction === 0 ? codec.decodeRequest : codec.decodeResponse;
  const encode = direction === 0 ? codec.encodeRequest : codec.encodeResponse;
  if (!accepted) assert.throws(() => decode(input), WireError);
  else assert.deepEqual(encode(decode(input)), input);
}

// Records are replayable data; keep assertion logic independent of generation.
async function checkMutations(codecs, expected) {
  const lines = createInterface({
    input: createReadStream(new URL('./mutations.jsonl', import.meta.url)),
    crlfDelay: Infinity,
  });
  let count = 0;
  for await (const line of lines) {
    count++;
    try {
      const { codec, direction, bytes, accepted } = JSON.parse(line);
      assert.ok(Object.hasOwn(codecs, codec), 'unknown mutation codec');
      assert.ok(direction === 0 || direction === 1, 'invalid mutation direction');
      assert.equal(typeof accepted, 'boolean');
      mutation(codecs[codec], direction, bytes, accepted);
    } catch (cause) {
      throw new Error(`mutation line ${count}`, { cause });
    }
  }
  assert.equal(count, expected, 'mutation record count');
}

const packedI16 = probe({k:'packed-i16',max:2});
assert.throws(() => packedI16.encodeRequest([1,2]), WireError);
assert.throws(() => packedI16.encodeRequest(new Int16Array(3)), WireError);
assert.throws(() => packedI16.decodeRequest(Uint8Array.from([...header,1,0])), WireError);
assert.throws(() => packedI16.decodeRequest(Uint8Array.from([...header,6,0,0,0,0,0,0])), WireError);
assert.throws(() => packedI16.decodeRequest(Uint8Array.from([...header,0x82,0,0,0])), WireError);
assert.deepEqual(packedI16.decodeRequest(Uint8Array.from([...header,4,0,128,255,127])),new Int16Array([-32768,32767]));

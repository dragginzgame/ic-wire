import assert from 'node:assert/strict';
import { WireError } from './compiled/wire-runtime.js';
const normalize = value => value instanceof Uint8Array ? value : Array.isArray(value) ? value.map(normalize) : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k,v]) => [k, normalize(v)])) : value;
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
  }
}
// The descriptor here exercises hostile inputs without duplicating Rust schemas.
import { createCodec } from './compiled/wire-runtime.js';
function probe(schema, nodes = 32, depth = 8) {
  return createCodec({id:'1',request:schema,response:schema,requestBytes:128,responseBytes:128,requestNodes:nodes,responseNodes:nodes,requestDepth:depth,responseDepth:depth});
}
const header = [73,87,1,0,0,0,0,0,0,0,0];
const frame = (...payload) => Uint8Array.from([...header,...payload]);
const rejects = (action, code) => assert.throws(action, error => error instanceof WireError && error.code === code);
for (const payload of [[128,0],[255,255,4],[128,128,128]]) rejects(() => probe({k:'u',bits:16}).decodeRequest(frame(...payload)), 'malformed');
rejects(() => probe({k:'bool'}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'option',item:{k:'unit'}}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'result',ok:{k:'unit'},err:{k:'unit'}}).decodeRequest(frame(2)), 'malformed');
rejects(() => probe({k:'enum',variants:[{name:'Unit',id:128,value:{k:'unit'}}]}).decodeRequest(frame(3)), 'malformed');
for (const payload of [[1,255],[2,192,128],[3,237,160,128],[1,239]]) rejects(() => probe({k:'text',max:8}).decodeRequest(frame(...payload)), 'malformed');
for (const k of ['bytes','text','list']) rejects(() => probe({k,max:4,item:{k:'unit'}}).decodeRequest(frame(5)), 'limit');
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
console.log('Finite-float canonical bits and rejection checks passed');

import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
let stdin = ''; for await (const chunk of process.stdin) stdin += chunk;
const fixture = JSON.parse(stdin), url = pathToFileURL(process.argv[1]);
const { WireError, createPayloadCodec } = await import(url);
const { packed, ordinary } = await import(new URL('./frame-client.ts', url));
const refusal = (action, code) => assert.throws(action, error => error instanceof WireError && error.code === code);
const packedPrefixes = [[0], [2], [128,1], [128,2], [128,16], [128,128,2]];
const listPrefixes = [[0], [1], [64], [128,1], [128,8], [128,128,1]];
const mixed = [-32768,-16385,-65,-64,-1,0,63,64,16384,32767];
const words = [[0,128],[255,191],[191,255],[192,255],[255,255],[0,0],[63,0],[64,0],[0,64],[255,127]];
const varints = [[255,255,3],[129,128,2],[129,1],[127],[1],[0],[126],[128,1],[128,128,2],[254,255,3]];
const output = [];
for (const [sizeIndex, size] of fixture.sizes.entries()) {
  for (let distribution = 0; distribution < 4; distribution++) {
    const values = [], packedBody = [], listBody = [];
    for (let i = 0; i < size; i++) {
      const n = i % 64, m = i % mixed.length;
      if (distribution === 0) { values.push(n); packedBody.push(n,0); listBody.push(n*2); }
      if (distribution === 1) { values.push(32767); packedBody.push(255,127); listBody.push(254,255,3); }
      if (distribution === 2) { values.push(-1-n); packedBody.push(255-n,255); listBody.push(1+n*2); }
      if (distribution === 3) { values.push(mixed[m]); packedBody.push(...words[m]); listBody.push(...varints[m]); }
    }
    const expectedPacked = Uint8Array.from([...packedPrefixes[sizeIndex], ...packedBody]);
    const expectedList = Uint8Array.from([...listPrefixes[sizeIndex], ...listBody]);
    const storage = new Int16Array(size + 2).fill(999), selected = storage.subarray(1,size+1); selected.set(values);
    for (const [codec, id, value, expected, schema, nodes, depth] of [
      [packed,7,selected,expectedPacked,{k:'packed-i16',max:16384},1,1],
      [ordinary,8,values,expectedList,{k:'list',item:{k:'i',bits:16},max:16384},size+1,2],
    ]) {
      const exact = createPayloadCodec({schema,bytes:expected.length,nodes,depth});
      assert.deepEqual(exact.encode(value),expected); assert.deepEqual(exact.decode(expected),value);
      if (nodes > 1) {
        const short = createPayloadCodec({schema,bytes:expected.length,nodes:nodes-1,depth});
        refusal(() => short.encode(value),'limit'); refusal(() => short.decode(expected),'limit');
      }
      const shortBytes = createPayloadCodec({schema,bytes:expected.length-1,nodes,depth});
      refusal(() => shortBytes.encode(value),'limit'); refusal(() => shortBytes.decode(expected),'limit');
      for (const [direction, encode, decode] of [[0,codec.encodeRequest,codec.decodeRequest],[1,codec.encodeResponse,codec.decodeResponse]]) {
        const frame = new Uint8Array(expected.length+11); frame.set([73,87,id,0,0,0,0,0,0,0,direction]); frame.set(expected,11);
        assert.deepEqual(encode(value),frame);
        // Selected subviews include unaligned packed bodies at these sizes.
        const input = new Uint8Array(frame.length+3); input.set(frame,1);
        const decoded = decode(input.subarray(1,frame.length+1));
        input.fill(255); assert.deepEqual(decoded,value);
      }
    }
    output.push([Array.from(expectedPacked),Array.from(expectedList)]);
  }
}
const codec = createPayloadCodec({schema:{k:'packed-i16',max:16384},bytes:32771,nodes:1,depth:1});
refusal(() => codec.decode(Uint8Array.of(1,0)),'malformed');
refusal(() => codec.encode(new Int16Array(16385)),'limit');
refusal(() => codec.encode([1,2]),'value');
assert.deepEqual(codec.decode(Uint8Array.of(128,0)),new Int16Array());
process.stdout.write(JSON.stringify(output));

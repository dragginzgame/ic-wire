// Matched JavaScript runtime timing; setup/validation excluded, allocations included.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const [beforePath, afterPath, output] = process.argv.slice(2);
const before = await import(pathToFileURL(beforePath));
const after = await import(pathToFileURL(afterPath));
const hash = value => createHash('sha256').update(value).digest('hex');
const cases = [
  ['unit', {k:'unit'}, null],
  ['u64', {k:'u',bits:64}, 7n],
  ['vec-8', {k:'list',item:{k:'u',bits:32}}, Array(8).fill(0xffffffff)],
  ['vec-4096', {k:'list',item:{k:'u',bits:32}}, Array(4096).fill(0xffffffff)],
  ['blob-32', {k:'bytes'}, new Uint8Array(32).fill(165)],
  ['blob-65536', {k:'bytes'}, new Uint8Array(65536).fill(165)],
  ['record', {k:'record',fields:[{name:'count',value:{k:'u',bits:64}},{name:'text',value:{k:'text'}}]}, {count:18446744073709551615n,text:'é🦀'}]
];
const rows = [];
let observed;
for (const [name, schema, value] of cases) {
  const meta = {id:'17',request:schema,response:schema,requestBytes:131072,responseBytes:131072,requestNodes:131072,responseNodes:131072,requestDepth:64,responseDepth:64};
  const old = before.createCodec(meta), framed = after.createCodec(meta);
  const payload = after.createPayloadCodec({schema,bytes:131061,nodes:131072,depth:64});
  const expected = old.encodeRequest(value);
  assert.deepEqual(framed.encodeRequest(value), expected);
  assert.deepEqual(payload.encode(value), expected.slice(11));
  const lanes = [
    {name:'before-framed',encode:v=>old.encodeRequest(v),decode:b=>old.decodeRequest(b),bytes:expected},
    {name:'after-framed',encode:v=>framed.encodeRequest(v),decode:b=>framed.decodeRequest(b),bytes:expected},
    {name:'after-payload',encode:v=>payload.encode(v),decode:b=>payload.decode(b),bytes:expected.slice(11)}
  ];
  const repetitions = expected.length > 1024 ? 64 : 4096;
  const results = lanes.map(lane => ({case:name,lane:lane.name,repetitions,encoded_bytes:lane.bytes.length,input_sha256:hash(expected),encode_ns:[],decode_ns:[]}));
  for (const lane of lanes) for (let i=0;i<256;i++) { observed=lane.encode(value); observed=lane.decode(lane.bytes); }
  for (let sample=0;sample<7;sample++) {
    for (let i=0;i<lanes.length;i++) {
      const index=(i+sample)%lanes.length, lane=lanes[index], row=results[index];
      let start=process.hrtime.bigint();
      for (let n=0;n<repetitions;n++) observed=lane.encode(value);
      row.encode_ns.push(Number(process.hrtime.bigint()-start));
      start=process.hrtime.bigint();
      for (let n=0;n<repetitions;n++) observed=lane.decode(lane.bytes);
      row.decode_ns.push(Number(process.hrtime.bigint()-start));
    }
  }
  rows.push(...results);
}
assert.notEqual(observed, undefined);
fs.writeFileSync(output, JSON.stringify({node:process.version,versions:process.versions,before_sha256:hash(fs.readFileSync(beforePath)),after_sha256:hash(fs.readFileSync(afterPath)),limits:{frame_bytes:131072,payload_bytes:131061,nodes:131072,depth:64},samples:7,rows},null,2));

import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
let stdin=''; for await(const chunk of process.stdin) stdin+=chunk;
const fixture=JSON.parse(stdin), url=pathToFileURL(process.argv[1]);
const {codec}=await import(new URL('./client.ts',url));
const {WireError}=await import(url);
const output=[];
for(const optional of [null,{some:new Uint8Array()},{some:Uint8Array.of(3,4)}]) {
  const value={plain:[0,255],exact:Uint8Array.of(0,255),buffer:Uint8Array.of(7,8),optional};
  const bytes=codec.encode(value); assert.deepEqual(Array.from(bytes),fixture.literals[output.length]);
  assert.deepEqual({...codec.decode(bytes)},value);
  const decoded=codec.decode(bytes); bytes.fill(0); assert.deepEqual({...decoded},value);
  output.push(fixture.literals[output.length]);
  assert.throws(()=>codec.encode({...value,exact:Uint8Array.of(1)}),e=>e instanceof WireError && e.code==='value');
  assert.throws(()=>codec.encode({...value,exact:Uint8Array.of(1,2,3)}),e=>e instanceof WireError && e.code==='limit');
}
process.stdout.write(JSON.stringify(output));

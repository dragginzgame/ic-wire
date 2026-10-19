{
  const codec = await import('./compiled/large.js');
  const { readFileSync } = await import('node:fs');
  const value = Uint8Array.from({length:1024*1024},(_,i)=>i%251);
  for (const [encode,decode,file] of [[codec.encodeRequest,codec.decodeRequest,'large-request.bin'],[codec.encodeResponse,codec.decodeResponse,'large-response.bin']]) {
    const expected=Uint8Array.from(readFileSync(new URL(file,import.meta.url)));
    assert.deepEqual(encode(value),expected);
    assert.deepEqual(decode(expected),value);
    for (const end of [0,10,11,12,Math.floor(expected.length/2),expected.length-1]) assert.throws(()=>decode(expected.slice(0,end)),WireError);
    const wrong=expected.slice(); wrong[2]^=1; assert.throws(()=>decode(wrong),WireError);
    const direction=expected.slice(); direction[10]^=1; assert.throws(()=>decode(direction),WireError);
    const trailing=new Uint8Array(expected.length+1); trailing.set(expected); assert.throws(()=>decode(trailing),WireError);
    const nonminimal=new Uint8Array([...expected.slice(0,11),128,0]); assert.throws(()=>decode(nonminimal),WireError);
    assert.throws(()=>encode(new Uint8Array(value.length+1)),WireError);
  }
  console.log('1 MiB Rust/TypeScript byte buffers: exact frames and boundary refusals passed');
}

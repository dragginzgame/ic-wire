// Observe byte-buffer requests, not JS heap or allocation latency.
{
  const Original = globalThis.Uint8Array;
  let requests;
  class ObservedBytes extends Original {
    constructor(...args) {
      super(...args);
      if (requests && !(args[0] instanceof ArrayBuffer)) requests.push(this.byteLength);
    }
    static [Symbol.hasInstance](value) { return value instanceof Original; }
  }
  const capture = operation => {
    requests = [];
    globalThis.Uint8Array = ObservedBytes;
    try { return {value: operation(), requests}; }
    finally { globalThis.Uint8Array = Original; requests = undefined; }
  };
  const codec = (schema, requestBytes = 2 * 1024 * 1024, responseBytes = 3 * 1024 * 1024, checks = {}) => createCodec({
    id:'123', request:schema, response:schema, requestBytes,responseBytes,
    requestNodes:10000,responseNodes:10000,requestDepth:32,responseDepth:32
  }, {}, checks);
  const unit = codec({k:'unit'});
  const huge = capture(()=>codec({k:'unit'},Number.MAX_SAFE_INTEGER,Number.MAX_SAFE_INTEGER).encodeRequest(null));
  assert.equal(huge.value.length,11);
  assert.ok(huge.requests.every(n=>n<=64));
  for (const direction of ['Request','Response']) {
    const success = capture(() => unit['encode'+direction](null));
    assert.equal(success.value.length,11);
    assert.ok(success.requests.every(n=>n<=64));
    const invalid = capture(() => assert.throws(()=>unit['encode'+direction]('invalid'), e=>e instanceof WireError && e.code==='value'));
    assert.ok(invalid.requests.every(n=>n<=64));
  }
  const cases = [
    [{k:'unit'},null], [{k:'bytes'},new Original(0)],
    [{k:'bytes'},new Original(4096).fill(42)], [{k:'text'},''],
    [{k:'text'},'😀é漢'.repeat(128)],
    [{k:'packed',word:'i16'},new Int16Array([0,-32768,32767])],
    [{k:'packed',word:'u64'},new BigUint64Array(1024).fill(18446744073709551615n)],
    [{k:'array',max:300,item:{k:'float',bits:64}},Array.from({length:300},(_,i)=>i+0.25)],
    [{k:'array',max:300,item:{k:'float',bits:32}},Array.from({length:300},(_,i)=>i+0.25)],
    [{k:'list',item:{k:'u',bits:32}},Array.from({length:4096},(_,i)=>i)],
  ];
  for (const [schema,value] of cases) {
    const roomy = codec(schema), expected = roomy.encodeRequest(value);
    for (const direction of ['Request','Response']) {
      const encode = 'encode'+direction, decode = 'decode'+direction;
      const exact = codec(schema,expected.length,expected.length);
      const observed = capture(()=>exact[encode](value));
      assert.ok(observed.requests.every(n=>n<=expected.length));
      assert.deepEqual([...observed.value],[...roomy[encode](value)]);
      assert.deepEqual(exact[decode](observed.value),value);
      if(expected.length>11) assert.throws(()=>codec(schema,expected.length-1,expected.length-1)[encode](value),e=>e instanceof WireError && e.code==='limit');
      // Results are independent across calls and never alias a caller's buffer.
      const retained = exact[encode](value), original = retained.slice();
      observed.value.fill(0);
      assert.deepEqual(retained,original);
      assert.deepEqual([...exact[encode](value)],[...original]);
    }
  }
  const asymmetric = codec({k:'bytes'},64,65), payload = new Original(53);
  assert.throws(()=>asymmetric.encodeRequest(payload),e=>e instanceof WireError && e.code==='limit');
  assert.equal(asymmetric.encodeResponse(payload).length,65);
  let calls=0;
  const checked = codec({k:'bytes',check:'once'},10000,10000,{once:()=>{calls++;}});
  const input = new Original(4096).fill(7), copy = input.slice();
  checked.encodeRequest(input); assert.equal(calls,1);
  checked.encodeResponse(input); assert.equal(calls,2);
  assert.deepEqual(input,copy);
  for(const schema of [{k:'text'},{k:'packed',word:'i16'},{k:'float',bits:64}]) {
    assert.throws(()=>codec(schema).encodeRequest({}),WireError);
  }
  assert.throws(()=>codec({k:'text'}).encodeRequest('\ud800'),e=>e instanceof WireError && e.code==='value');
  console.log('Incremental writer: bounded allocations, exact bytes, ownership, callbacks and directional limits passed');
}

// Collection-length fast paths retain u32 overflow and error precedence.
{
  const descriptor = schema => ({id:'1',request:schema,response:schema,
    requestBytes:1048576,responseBytes:1048576,requestNodes:1048576,
    responseNodes:1048576,requestDepth:32,responseDepth:32});
  const header = [73,87,1,0,0,0,0,0,0,0,0];
  const varint = n => {
    const bytes=[];
    do { const byte=Number(n & 127n); n >>= 7n; bytes.push(n ? byte | 128 : byte); } while(n);
    return bytes;
  };
  for (const n of [0,1,127,128,16383,16384,65536]) {
    const bytes = new Uint8Array(n).fill(42), codec=createCodec(descriptor({k:'bytes'}));
    const encoded=codec.encodeRequest(bytes), prefix=varint(BigInt(n));
    assert.deepEqual([...encoded.subarray(11,11+prefix.length)],prefix);
    assert.deepEqual(codec.decodeRequest(encoded),bytes);
  }
  for (const schema of [{k:'bytes',max:0},{k:'text',max:0},{k:'list',max:0,item:{k:'u',bits:8}}]) {
    const codec=createCodec(descriptor(schema));
    assert.throws(()=>codec.decodeRequest(Uint8Array.from([...header,...varint(0xffffffffn)])),e=>e instanceof WireError && e.code==='limit');
    for (const prefix of [[128],[128,128,128,128,128],varint(0x100000000n),[255,255,255,255,127]]) {
      assert.throws(()=>codec.decodeRequest(Uint8Array.from([...header,...prefix])),e=>e instanceof WireError && e.code==='malformed');
    }
    const empty=codec.decodeRequest(Uint8Array.from([...header,128,0]));
    assert.equal(empty.length,0); // Upstream Postcard permits redundant zero groups.
  }
}

// Packed bool conversion validates the entire span before allocating its result.
{
  const codec=createCodec({id:'1',request:{k:'packed',word:'bool'},response:{k:'packed',word:'bool'},
    requestBytes:1048576,responseBytes:1048576,requestNodes:32,responseNodes:32,
    requestDepth:8,responseDepth:8});
  const OriginalArray=globalThis.Array;
  let allocations=0;
  class ObservedArray extends OriginalArray {
    constructor(...args) { super(...args); allocations++; }
  }
  for(const n of [0,1,7,8,9,127,128,129]) {
    const values=OriginalArray.from({length:n},(_,i)=>i%2===0);
    for(const direction of ['Request','Response']) {
      const frame=codec['encode'+direction](values);
      const decoded=codec['decode'+direction](frame);
      assert.deepEqual(decoded,values);
      frame.fill(255);
      assert.deepEqual(decoded,values); // Owned, not a view into the input.
    }
  }
  const frame=codec.encodeRequest(OriginalArray(129).fill(false));
  for(const position of [0,7,8,127,128]) for(let byte=2;byte<256;byte++) {
    const malformed=frame.slice(); malformed[13+position]=byte; // 11-byte header + two-byte length.
    allocations=0;
    let error;
    globalThis.Array=ObservedArray;
    try { codec.decodeRequest(malformed); } catch (caught) { error=caught; }
    finally { globalThis.Array=OriginalArray; }
    assert.ok(error instanceof WireError && error.code==='malformed');
    assert.equal(allocations,0);
  }
}
